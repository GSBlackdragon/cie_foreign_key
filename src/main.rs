mod fk_relation;
pub mod db_config;

use fk_relation::FkRelation;
use db_config::DatabaseConfig;

use tiberius::{Client, Config, AuthMethod};
use tokio::net::TcpStream;
use tokio_util::compat::{TokioAsyncWriteCompatExt};
use tokio_postgres::{NoTls};
use clap::{Parser};
use log::{LevelFilter};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use chrono::Utc;

// Args and parameters the program can handle
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    clear: bool
}

/// Return a log4rs Config struct matching the needs of the application, ready to be used.
fn log_config_creation() -> log4rs::config::Config{
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} : {d(%Y-%m-%d %H:%M:%S)} - {m}\n")))
        .build(format!("logs/{}.log", Utc::now().format("%Y-%m-%dT%H.%M.%S"))).expect("Error occurred while creating logfile");

    return log4rs::config::Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder()
            .appender("logfile")
            .build(LevelFilter::Info)).expect("Error while building log4rs Config struct");
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    // Load config
    let dbs_config: DatabaseConfig = confy::load_path("config.conf")?;

    // Loading log config
    let log_config = log_config_creation();

    log4rs::init_config(log_config)?;

    // Get args
    let args = Args::parse();

    // SQL Server connection
    let mut config = Config::new();

    config.host(dbs_config.sql_server_host);
    config.port(dbs_config.sql_server_port);
    config.application_name("CIEForeignKey");
    config.authentication(AuthMethod::sql_server(dbs_config.sql_server_user, dbs_config.sql_server_pass));
    config.database(dbs_config.sql_server_db);
    config.trust_cert();

    let tcp = match TcpStream::connect(config.get_addr()).await {
        Ok(tcp) => {
            log::info!("TCP Stream connected to {}.", config.get_addr());
            tcp
        },
        Err(error) => {
            log::error!("TCP connection to {} failed : {}.", config.get_addr(), error);
            panic!("{:?}", error);
        }
    };
    tcp.set_nodelay(true)?;

    let mut sqlserver_client = match Client::connect(config, tcp.compat_write()).await {
        Ok(client) => {
            log::info!("Connected to SQL Server.");
            client
        },
        Err(error) => {
            log::error!("Connection to SQL Server failed : {}.", error);
            panic!("{}", error)
        }
    };

    // Postgres connection
    let postgres_connection_string = format!("host={} port={} user={} password={} dbname={}", dbs_config.postgres_host, dbs_config.postgres_port, dbs_config.postgres_user, dbs_config.postgres_pass, dbs_config.postgres_db);

    let (postgres_client, connection) = match tokio_postgres::connect(postgres_connection_string.as_str(), NoTls).await {
        Ok(client_conn) => {
            log::info!("Connected to postgres.", );
            client_conn
        },
        Err(error) => {
            log::error!("Connection to postgres failed : {}.", error);
            panic!("{}", error)
        }
    };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Check for -c arg and if present, remove all FKey then return
    if args.clear {

        let clear_select_result = sqlserver_client.query("SELECT CONSTRAINT_NAME, TABLE_NAME FROM INFORMATION_SCHEMA.TABLE_CONSTRAINTS WHERE CONSTRAINT_TYPE = 'FOREIGN KEY';", &[]).await?.into_results().await?;
        let mut removed_constraint_count = 0;
        for row in &clear_select_result[0] {
            let constraint_name: &str = match row.get(0) {
              Some(str) => str,
                _ => ""
            };

            let table_name: &str = match row.get(1) {
                Some(str) => str,
                _ => ""
            };

            if !(constraint_name.len() == 0 || table_name.len() == 0) {
                let clear_query = format!("ALTER TABLE {} DROP CONSTRAINT {};", table_name, constraint_name);
                match sqlserver_client.query(clear_query, &[]).await {
                    Ok(_) => {removed_constraint_count += 1}
                    Err(error) => {log::error!("Table {} constraint {} failed to drop. Error : {}", table_name, constraint_name, error)}
                };
            }
        }
        log::info!("{} foreign key have been removed.", removed_constraint_count);
        log::info!("Program end.");
        return Ok(());
    }

    // Recover sql server tables
    let sql_server_table_result = &sqlserver_client.query("SELECT TABLE_NAME FROM information_schema.TABLES WHERE TABLE_TYPE = 'BASE TABLE';", &[]).await?.into_results().await?[0];
    let mut sql_server_table_name_vec: Vec<&str> = vec![];

    for row in sql_server_table_result {
        if let Some(result) = row.get(0) {
            sql_server_table_name_vec.push(result);
        }
    }

    // Recover foreign key for tables in table_name_vec and updates relations for it
    let mut success_relation_count = 0;
    for table in &sql_server_table_name_vec {
        let mut relations: Vec<FkRelation> = vec![];

        let recover_fk_sql_query = "
            SELECT
                kcu.constraint_name,
                kcu.table_name,
                kcu.column_name,
                kcu1.table_name AS references_table,
                kcu1.column_name AS references_field
            FROM information_schema.key_column_usage kcu
                LEFT JOIN information_schema.referential_constraints rc ON kcu.constraint_name = rc.constraint_name
                LEFT JOIN information_schema.key_column_usage kcu1 ON rc.unique_constraint_name = kcu1.constraint_name
            WHERE kcu.table_name = $1
                AND kcu.constraint_name LIKE '%fkey';";
        let rows = postgres_client.query(recover_fk_sql_query, &[&table]).await?;

        for row in rows {
            let constraint_name: String = row.get(0);
            let table_name: String = row.get(1);
            let column_name: String = row.get(2);
            let references_table: String = row.get(3);
            let references_column: String = row.get(4);

            if sql_server_table_name_vec.contains(&&**&references_table) {
                relations.push(FkRelation{constraint_name, table_name, column_name, references_table, references_column});
            }
        }

        for relation in &relations {
            let res = sqlserver_client.query(relation.get_fkey_constraint(), &[]).await;
            if let Ok(_) = res {
                success_relation_count += 1;
            }
        }
    }

    log::info!("{} foreign key created", {success_relation_count});
    log::info!("Program end");
    Ok(())
}