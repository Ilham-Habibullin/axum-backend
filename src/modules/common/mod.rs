use tokio_postgres::types::ToSql;

#[derive(serde::Deserialize)]
pub struct Search {
    pub search: Option<String>,
}

pub type QuieryBuildParam = (&'static str, Box<(dyn ToSql + Sync + Send)>);

pub type SqlParams = Vec<QuieryBuildParam>;
