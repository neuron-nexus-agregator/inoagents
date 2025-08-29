use crate::db::model::Record;
use serde::Serialize;

pub struct RecordWithRelevance {
    pub record: Record,
    pub similarity: f32,
}

#[derive(Serialize, Debug, Clone)]
pub struct Distances {
    #[serde(rename = "orig_name_dis")]
    pub not_normal_dis: usize,
    #[serde(rename = "norm_name_dis")]
    pub normal_dis: usize,
    #[serde(rename = "orig_name_dis_2")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_dis: Option<usize>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Doc {
    pub status: String,
    pub name: String,
    pub is_removed: bool,
    pub similarity: f32,
    pub distance: usize,
    pub debug_distances: Option<Distances>,
}

#[derive(Debug, Serialize)]
pub struct WarningName {
    pub name: String,
    pub normal_name: String,
    pub context: String,
    pub name_type: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub docs: Vec<Doc>,
}

#[derive(Debug, Serialize)]
pub struct WarningNames {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<WarningName>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub accepted_names: Vec<WarningName>,
}
