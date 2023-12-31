use std::{cell::RefCell, rc::Rc, sync::RwLock};

use rustyline::completion::{Candidate, Completer};

use super::context::Context;

pub struct YspCompleter {
    connection: Rc<RwLock<Context>>,
    tables: RefCell<Vec<String>>,
    views: RefCell<Vec<String>>,
}

impl YspCompleter {
    pub fn new(connection: Rc<RwLock<Context>>) -> Self {
        Self {
            connection,
            tables: Default::default(),
            views: Default::default(),
        }
    }
}

impl Completer for YspCompleter {
    type Candidate = YspCandidate;

    fn complete(
        &self, // FIXME should be `&mut self`
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let _ = (line, pos, ctx);
        let mut results = vec![];
        if let Some(trailing) = line
            .trim_end_matches(';')
            .to_uppercase()
            .strip_prefix("SELECT * FROM ")
        {
            if let Some((tables_or_views, existed_columns)) = trailing.split_once("WHERE") {
                for table_or_view in tables_or_views.split(',').map(|x| x.trim()) {
                    results.extend(self.get_columns(
                        table_or_view,
                        existed_columns.split([' ']).last().unwrap_or_default(),
                    ));
                }
            } else {
                results.extend(self.complete_query(trailing));
            }
        } else if let Some(trailing) = line
            .trim_end_matches(';')
            .to_uppercase()
            .strip_prefix("DESC ")
        {
            results.extend(self.complete_query(trailing));
        } else {
            results.extend(
                keywords()
                    .iter()
                    .map(|x| x.to_string())
                    .map(YspCandidate::Keyword),
            );
        }

        Ok((pos, results))
    }

    fn update(
        &self,
        line: &mut rustyline::line_buffer::LineBuffer,
        start: usize,
        elected: &str,
        cl: &mut rustyline::Changeset,
    ) {
        let end = line.pos();
        line.replace(start..end, elected, cl);
    }
}

impl YspCompleter {
    pub fn complete_query(&self, trailing: &str) -> Vec<YspCandidate> {
        let mut results = vec![];
        if self.tables.take().is_empty() {
            self.tables
                .replace(self.get_tables_or_views("select table_name from dba_tables"));
        }
        if self.views.take().is_empty() {
            self.views
                .replace(self.get_tables_or_views("select view_name from dba_views"));
        }

        let prefix = trailing
            .split(' ')
            .last()
            .unwrap_or_default()
            .to_uppercase();
        results.extend(
            self.tables
                .borrow()
                .iter()
                .filter(|x| x.starts_with(&prefix))
                .map(|x| YspCandidate::Table(x[prefix.len()..].to_string())),
        );
        results.extend(
            self.views
                .take()
                .iter()
                .filter(|x| x.starts_with(&prefix))
                .map(|x| YspCandidate::View(x[prefix.len()..].to_string())),
        );
        results
    }

    fn get_tables_or_views(&self, sql: &str) -> Vec<String> {
        if let Some(connection) = self.connection.read().unwrap().get_connection() {
            if let Ok(stmt) = connection.create_statement() {
                if let Ok(result) = stmt.execute_sql(sql) {
                    return result
                        .result_set()
                        .rows()
                        .map(|r| match r[0].as_ref().unwrap() {
                            yasqlplus_client::wrapper::Value::VarChar(t) => t.clone(),
                            _ => unreachable!(),
                        })
                        .collect::<Vec<_>>();
                }
            }
        }
        vec![]
    }

    fn get_columns(&self, table_or_view: &str, prefix: &str) -> Vec<YspCandidate> {
        if let Some(connection) = self.connection.read().unwrap().get_connection() {
            if let Ok(stmt) = connection.create_statement() {
                if let Ok(result) =
                    stmt.execute_sql(&format!("select * from {table_or_view} where 1 = 2"))
                {
                    return result
                        .result_set()
                        .iter_columns()
                        .map(|r| r.name)
                        .filter(|x| x.starts_with(prefix))
                        .map(|x| x[prefix.len()..].to_string())
                        .map(YspCandidate::Column)
                        .collect::<Vec<_>>();
                }
            }
        }
        vec![]
    }
}

pub fn keywords() -> Vec<&'static str> {
    // TODO
    vec!["SELECT", "CREATE", "INSERT"]
}

pub enum YspCandidate {
    Keyword(String),
    Table(String),
    View(String),
    Column(String),
}

impl Candidate for YspCandidate {
    fn display(&self) -> &str {
        match self {
            YspCandidate::Keyword(v) => v,
            YspCandidate::Table(v) => v,
            YspCandidate::View(v) => v,
            YspCandidate::Column(v) => v,
        }
    }

    fn replacement(&self) -> &str {
        match self {
            YspCandidate::Keyword(v)
            | YspCandidate::Table(v)
            | YspCandidate::View(v)
            | YspCandidate::Column(v) => v,
        }
    }
}
