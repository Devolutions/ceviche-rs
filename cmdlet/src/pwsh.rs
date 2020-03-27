
use std::path::{PathBuf};
use std::process::{Command};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct PSModuleManifest {
    #[serde(rename = "ModuleVersion")]
    pub module_version: String,
    #[serde(rename = "CompanyName")]
    pub company_name: String,
    #[serde(rename = "Description")]
    pub description: String,
}

pub fn find_powershell() -> Option<PathBuf> {
    if let Ok(powershell) = which::which("pwsh") {
        return Some(powershell);
    }
    which::which("powershell").ok()
}

pub fn find_cmdlet_base(module_name: &str) -> Option<PathBuf> {
    let powershell = find_powershell()?;

    let command = format!(
        "Get-Module -Name {} -ListAvailable | Select-Object -First 1 | % ModuleBase",
        module_name);

    let output = Command::new(&powershell)
        .arg("-Command").arg(&command)
        .output().ok()?;

    let module_base = String::from_utf8(output.stdout).ok()?;
    return Some(PathBuf::from(module_base.trim()));
}

pub fn get_module_manifest(module_name: &str) -> Option<PSModuleManifest> {
    let powershell = find_powershell()?;
    let manifest_path = find_cmdlet_base(module_name)?;
    let manifest_path = manifest_path.as_path().to_str()?;

    let command = format!(
        "Import-PowerShellDataFile -Path \"{}\\{}.psd1\" | ConvertTo-Json",
        manifest_path, module_name);

    let output = Command::new(&powershell)
        .arg("-Command").arg(&command)
        .output().ok()?;

    let json_output = String::from_utf8(output.stdout).ok()?;
    serde_json::from_str(json_output.as_str()).ok()
}
