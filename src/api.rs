use reqwest;
use std::borrow::Borrow;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultsRoot {
    pub version: i64,
    pub limit: i64,
    pub valid: bool,
    pub results: Vec<PackageDetails>,
    #[serde(rename = "num_pages")]
    pub num_pages: i64,
    pub page: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageDetails {
    pub pkgname: String,
    pub pkgbase: String,
    pub repo: String,
    pub arch: String,
    pub pkgver: String,
    pub pkgrel: String,
    pub epoch: i64,
    pub pkgdesc: String,
    pub url: String,
    pub filename: String,
    #[serde(rename = "compressed_size")]
    pub compressed_size: i64,
    #[serde(rename = "installed_size")]
    pub installed_size: i64,
    #[serde(rename = "build_date")]
    pub build_date: String,
    #[serde(rename = "last_update")]
    pub last_update: String,
    #[serde(rename = "flag_date")]
    pub flag_date: Value,
    pub maintainers: Vec<String>,
    pub packager: String,
    pub groups: Vec<Value>,
    pub licenses: Vec<String>,
    pub conflicts: Vec<String>,
    pub provides: Vec<String>,
    pub replaces: Vec<Value>,
    pub depends: Vec<String>,
    pub optdepends: Vec<String>,
    pub makedepends: Vec<String>,
    pub checkdepends: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageFiles {
    pub pkgname: String,
    pub repo: String,
    pub arch: String,
    #[serde(rename = "pkg_last_update")]
    pub pkg_last_update: String,
    #[serde(rename = "files_last_update")]
    pub files_last_update: String,
    #[serde(rename = "files_count")]
    pub files_count: i64,
    #[serde(rename = "dir_count")]
    pub dir_count: i64,
    pub files: Vec<String>,
}

/// Preforms a loose search (name, description mentions) for a package
pub async fn search_packages_loose(pkg_name: &str) -> Result<Vec<PackageDetails>, String>
{
    let search_results : SearchResultsRoot = reqwest::Client::new()
        .get(format!("https://archlinux.org/packages/search/json/?q={}", pkg_name)).send()
        .await.unwrap()
        .json().await.unwrap();

    let search_results_vec : Vec<PackageDetails> = search_results.results;
    
    if search_results_vec.is_empty()
    {
        return Err(format!("No results found for {}", pkg_name));
    }

    return Ok(search_results_vec);
}

/// Preforms an exact search for a package 
pub async fn search_packages_exact(pkg_name: &str) -> PackageDetails
{
    let results : SearchResultsRoot = reqwest::get(format!("https://archlinux.org/packages/search/json/?name={}", pkg_name))
        .await.unwrap().json().await.unwrap();

    return results.results[0].to_owned();
}

pub async fn get_package_files(pkg: &PackageDetails) -> Vec<String>
{
    let api_url : String = 
        format!("https://archlinux.org/packages/{}/{}/{}/files/json", pkg.repo, pkg.arch, pkg.pkgname);
    
    let mut file_list : Vec<String> = Vec::new();

    let files : PackageFiles = reqwest::get(&api_url).await.unwrap().json().await.unwrap();

    for x in files.files {file_list.push(x)}

    return file_list
}