use std::fmt::format;
use std::process::exit;
use tiberius::{Client, Config, Query, AuthMethod, FromSqlOwned, Row};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use tokio_postgres::{NoTls, Error};
use tokio_postgres::types::ToSql;

struct FkRelation {
    constraint_name: String,
    table_name: String,
    column_name: String,
    references_table: String,
    references_column: String
}

impl FkRelation {
    fn get_fkey_constraint(&self) -> String {
        return format!("ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({});", self.table_name, self.constraint_name, self.column_name, self.references_table, self.references_column);

    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{

    //SQL Server connection
    let mut config = Config::new();

    config.host("CLD-VM-CEMINEU9");
    config.port(61738);
    config.application_name("CIEForeignKey");
    config.authentication(AuthMethod::sql_server("Inside", "Ins!de@44"));
    config.database("INSIDE_GEN");
    config.trust_cert();

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let mut sqlserver_client = Client::connect(config, tcp.compat_write()).await?;

    // Postgres connection
    let (postgres_client, connection) = tokio_postgres::connect("host=localhost port=5433 user=postgres password=root dbname=odoo_backup", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Recover sql server tables

    let res  = &sqlserver_client.query("SELECT TABLE_NAME FROM information_schema.TABLES WHERE TABLE_TYPE = 'BASE TABLE';", &[]).await?.into_results().await?[0];
    let mut table_name_vec: Vec<&str> = vec![];

    for row in res {
        let val: Option<&str> = row.get(0);
        if let Some(result) = val {
            table_name_vec.push(result);
        }
    }

    println!("{:?}", table_name_vec);

    // Recover foreign key for tables in table_name_vec and updates relations for it

    for table in &table_name_vec {

        let mut relations: Vec<FkRelation> = vec![];

        let sql_query = "
            SELECT
                kcu.constraint_name,
                kcu.table_name,
                kcu.column_name,
                kcu1.table_name AS references_table,
                kcu1.column_name AS references_field
            FROM information_schema.key_column_usage kcu
                LEFT JOIN information_schema.referential_constraints rc ON kcu.constraint_name = rc.constraint_name
                LEFT JOIN information_schema.key_column_usage kcu1 ON rc.unique_constraint_name = kcu1.constraint_name
            WHERE kcu.table_name = 'account_account'
                AND kcu.constraint_name LIKE '%fkey';";
        let rows = postgres_client.query(sql_query, &[]).await?;

        for row in rows {
            let constraint_name: String = row.get(0);
            let table_name: String = row.get(1);
            let column_name: String = row.get(2);
            let references_table: String = row.get(3);
            let references_column: String = row.get(4);

            if table_name_vec.contains(&&**&references_table) {
                relations.push(FkRelation{constraint_name, table_name, column_name, references_table, references_column})
            }
        }

        for relation in &relations {
            let _res = sqlserver_client.query(relation.get_fkey_constraint(), &[]).await;
        }

        break
    }

    Ok(())
}
