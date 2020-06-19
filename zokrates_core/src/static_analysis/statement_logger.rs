use crate::typed_absy::folder::Folder;
use crate::typed_absy::*;
use zokrates_field::Field;

pub struct StatementLogger;

impl StatementLogger {
    fn new() -> Self {
        StatementLogger {}
    }
    pub fn create_logs<T: Field>(p: TypedProgram<T>) -> TypedProgram<T> {
        StatementLogger::new().fold_program(p)
    }
}

impl<'ast, T: Field> Folder<'ast, T> for StatementLogger {
    fn fold_statement(&mut self, s: TypedStatement<'ast, T>) -> Vec<TypedStatement<'ast, T>> {
        return vec![TypedStatement::Log(format!("{}", s)), s];
    }
}
