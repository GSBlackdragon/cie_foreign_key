use std::fmt;

// Struct representing a Foreign Key Relation
pub struct FkRelation {
    pub constraint_name: String,
    pub table_name: String,
    pub column_name: String,
    pub references_table: String,
    pub references_column: String
}

// Implementation of functions for the FkRelation struct
impl FkRelation {
    // get struct value and return a SQL command that create the foreign key constraint
    pub fn get_fkey_constraint(&self) -> String {
        return format!("ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({});", self.table_name, self.constraint_name, self.column_name, self.references_table, self.references_column);
    }
}

// Implementation of Debug for FkRelation
impl fmt::Debug for FkRelation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "constraint_name : {}, table_name : {}, column_name : {}, references_table : {}, references_column : {}", self.constraint_name, self.table_name, self.column_name, self.references_table, self.references_column)
    }
}