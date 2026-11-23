// Used by the notch's build script (`#[path]`) and tested from the notch crate: reads the identity of the sparse package
// (`installer/sparse/AppxManifest.xml`) and writes the exe manifest that claims it.

/// The three values Windows matches between the sparse package and the `<msix>` element of `foldline.exe`.
#[derive(Debug, PartialEq, Eq)]
pub struct SparseIdentity {
    pub package_name: String,
    pub publisher: String,
    pub application_id: String,
}

/// The raw (still XML-escaped) value of `name` in the opening tag that starts at `<element `.
fn attribute(manifest: &str, element: &str, name: &str) -> Option<String> {
    let start = manifest.find(&format!("<{element}"))?;
    let after_name = &manifest[start + element.len() + 1..];
    if !after_name.starts_with(char::is_whitespace) {
        return attribute(&manifest[start + 1..], element, name);
    }
    let tag = &after_name[..after_name.find('>')?];
    let marker = format!("{name}=\"");
    let mut search_from = 0;
    while let Some(offset) = tag[search_from..].find(&marker) {
        let position = search_from + offset;
        // `Name="` also ends `DisplayName="`; only a whole attribute name counts.
        if tag[..position].ends_with(char::is_whitespace) {
            let value = &tag[position + marker.len()..];
            return Some(value[..value.find('"')?].to_owned());
        }
        search_from = position + marker.len();
    }
    None
}

/// The package name and publisher of `<Identity>` and the id of the first `<Application>`.
pub fn read_identity(manifest: &str) -> Option<SparseIdentity> {
    Some(SparseIdentity {
        package_name: attribute(manifest, "Identity", "Name")?,
        publisher: attribute(manifest, "Identity", "Publisher")?,
        application_id: attribute(manifest, "Application", "Id")?,
    })
}

/// The application manifest embedded in `foldline.exe`. Without the package registered, Windows ignores `<msix>` and the
/// program runs without identity.
pub fn exe_manifest(identity: &SparseIdentity) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
         <assembly xmlns=\"urn:schemas-microsoft-com:asm.v1\" manifestVersion=\"1.0\">\n  \
         <assemblyIdentity version=\"1.0.0.0\" name=\"{package_name}\" />\n  \
         <msix xmlns=\"urn:schemas-microsoft-com:msix.v1\" publisher=\"{publisher}\" packageName=\"{package_name}\" \
         applicationId=\"{application_id}\" />\n\
         </assembly>\n",
        package_name = identity.package_name,
        publisher = identity.publisher,
        application_id = identity.application_id,
    )
}

#[cfg(test)]
mod tests {
    use super::{SparseIdentity, exe_manifest, read_identity};

    const MANIFEST: &str = r#"<Package xmlns="x">
  <Identity
    Name="Foldline.Desktop"
    Publisher="CN=Example, O=Example &amp; Co, C=US"
    Version="{{VERSION}}" />
  <Properties><DisplayName>Foldline</DisplayName></Properties>
  <Applications>
    <Application Id="Foldline" Executable="foldline.exe">
      <uap:VisualElements DisplayName="Foldline" />
    </Application>
  </Applications>
</Package>"#;

    fn example_identity() -> SparseIdentity {
        SparseIdentity {
            package_name: "Foldline.Desktop".to_owned(),
            publisher: "CN=Example, O=Example &amp; Co, C=US".to_owned(),
            application_id: "Foldline".to_owned(),
        }
    }

    #[test]
    fn the_identity_and_application_id_are_read_from_the_package_manifest() {
        assert_eq!(read_identity(MANIFEST), Some(example_identity()));
    }

    #[test]
    fn display_name_is_not_taken_for_the_package_name() {
        let manifest = r#"<Identity DisplayName="Wrong" Name="Right" Publisher="CN=P" /><Application Id="A">"#;
        assert_eq!(read_identity(manifest).map(|identity| identity.package_name).as_deref(), Some("Right"));
    }

    #[test]
    fn the_applications_element_is_not_taken_for_an_application() {
        let manifest = r#"<Identity Name="N" Publisher="CN=P" /><Applications><Application Id="Real">"#;
        assert_eq!(read_identity(manifest).map(|identity| identity.application_id).as_deref(), Some("Real"));
    }

    #[test]
    fn a_manifest_without_an_application_has_no_identity() {
        assert_eq!(read_identity(r#"<Identity Name="N" Publisher="CN=P" />"#), None);
    }

    #[test]
    fn the_exe_manifest_claims_the_same_package_publisher_and_application() {
        let text = exe_manifest(&example_identity());
        let expected = r#"<msix xmlns="urn:schemas-microsoft-com:msix.v1" publisher="CN=Example, O=Example &amp; Co, C=US" packageName="Foldline.Desktop" applicationId="Foldline" />"#;
        assert!(text.contains(expected), "{text}");
    }
}
