use super::DeckError;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant, UNIX_EPOCH};

const TIMEOUT: Duration = Duration::from_secs(90);
static CONVERSION: Mutex<()> = Mutex::new(());

/// Cache identity used by Kotlin WordDocumentPdf: canonical path, mtime in milliseconds and length.
pub fn document_pdf_cache_file(path: impl AsRef<Path>) -> Result<PathBuf, DeckError> {
    let source = dunce::canonicalize(path)?;
    let metadata = fs::metadata(&source)?;
    let millis = match metadata.modified()?.duration_since(UNIX_EPOCH) {
        Ok(time) => time.as_millis() as i128,
        Err(error) => -(error.duration().as_nanos().div_ceil(1_000_000) as i128),
    };
    let identity = format!("{}:{}:{}", source.display(), millis, metadata.len());
    let directory = format!("{:x}", Sha256::digest(identity.as_bytes()));
    let home = env::home_dir()
        .ok_or_else(|| DeckError::Renderer("Cannot locate the user's home directory".into()))?;
    Ok(home
        .join(".churchpresenter")
        .join("document-pdf")
        .join(directory)
        .join("document.pdf"))
}

fn libreoffice() -> Option<PathBuf> {
    let mut directories = Vec::new();
    #[cfg(windows)]
    for variable in ["ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(path) = env::var_os(variable) {
            directories.push(PathBuf::from(path).join("LibreOffice/program"));
        }
    }
    #[cfg(target_os = "macos")]
    directories.push(PathBuf::from(
        "/Applications/LibreOffice.app/Contents/MacOS",
    ));
    if let Some(path) = env::var_os("PATH") {
        directories.extend(env::split_paths(&path));
    }
    for dir in directories {
        for name in if cfg!(windows) {
            &["soffice.com", "soffice.exe"][..]
        } else {
            &["soffice"][..]
        } {
            let path = dir.join(name);
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

pub(super) fn run_renderer(mut command: Command, name: &str) -> Result<(), DeckError> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| DeckError::Renderer(format!("Cannot start {name}: {e}")))?;
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return if status.success() {
                Ok(())
            } else {
                Err(DeckError::Renderer(format!("{name} failed ({status})")))
            };
        }
        if start.elapsed() >= TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return Err(DeckError::Renderer(format!(
                "{name} timed out after 90 seconds"
            )));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn export(source: &Path, work: &Path) -> Result<PathBuf, DeckError> {
    if let Some(office) = libreoffice() {
        let profile = url::Url::from_directory_path(work.join("profile"))
            .map_err(|_| DeckError::Renderer("Invalid LibreOffice profile directory".into()))?;
        let mut command = Command::new(office);
        command
            .arg(format!("-env:UserInstallation={profile}"))
            .args(["--headless", "--convert-to", "pdf", "--outdir"])
            .arg(work)
            .arg(source);
        run_renderer(command, "LibreOffice")?;
        return Ok(work
            .join(source.file_name().unwrap_or_default())
            .with_extension("pdf"));
    }
    export_with_com(source, work)
}

#[cfg(windows)]
fn export_with_com(source: &Path, work: &Path) -> Result<PathBuf, DeckError> {
    const EXPORT: &str = r#"
$ErrorActionPreference = 'Stop'
$office = $null
$document = $null
try {
    try { $office = New-Object -ComObject Word.Application }
    catch { $office = New-Object -ComObject KWPS.Application }
    $office.Visible = $false
    $office.DisplayAlerts = 0
    $office.AutomationSecurity = 3
    $document = $office.Documents.Open($env:CHURCHPRESENTER_WORD_SOURCE, $false, $true)
    $document.ExportAsFixedFormat($env:CHURCHPRESENTER_WORD_PDF, 17)
} finally {
    try { if ($null -ne $document) { $document.Close(0) } }
    finally { if ($null -ne $office) { $office.Quit() } }
}
"#;
    let pdf = work.join("document.pdf");
    let mut command = Command::new("powershell.exe");
    command
        .args(["-NoProfile", "-NonInteractive", "-STA", "-Command", EXPORT])
        .env("CHURCHPRESENTER_WORD_SOURCE", source)
        .env("CHURCHPRESENTER_WORD_PDF", &pdf);
    run_renderer(command, "Word/WPS PDF export")?;
    Ok(pdf)
}

#[cfg(not(windows))]
fn export_with_com(_source: &Path, _work: &Path) -> Result<PathBuf, DeckError> {
    Err(DeckError::Renderer(
        "Install LibreOffice to convert Word documents".into(),
    ))
}

/// Converts one Word document, publishing the PDF only after the renderer succeeds.
pub fn convert_word_to_pdf(path: impl AsRef<Path>) -> Result<PathBuf, DeckError> {
    let _lock = CONVERSION
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let source = dunce::canonicalize(path)?;
    let result = document_pdf_cache_file(&source)?;
    if fs::metadata(&result).is_ok_and(|m| m.is_file() && m.len() > 0) {
        return Ok(result);
    }
    let cache = result.parent().unwrap();
    fs::create_dir_all(cache)?;
    let work = tempfile::Builder::new()
        .prefix("conversion-")
        .tempdir_in(cache)?;
    let pdf = export(&source, work.path())?;
    if !fs::metadata(&pdf).is_ok_and(|m| m.is_file() && m.len() > 0) {
        return Err(DeckError::Renderer("Office did not produce a PDF".into()));
    }
    fs::rename(pdf, &result)?;
    Ok(result)
}
