use super::*;

const SECRET_B32: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
const SECRET_BYTES: &[u8] = b"12345678901234567890";

fn import(text: &str) -> ImportBatch {
    import_text(text, Vendor::Other).unwrap()
}

fn first(batch: &ImportBatch) -> &NewAccount {
    &batch.accounts[0]
}

#[test]
fn rejects_text_that_is_no_known_format() {
    for text in ["", "hello", "https://example.com", "[1, 2, 3]", "{\"unrelated\": true}"] {
        assert_eq!(import_text(text, Vendor::Other), Err(ImportError::UnsupportedFormat), "{text}");
    }
}

#[test]
fn refuses_input_far_larger_than_any_export() {
    let huge = " ".repeat(MAX_IMPORT_TEXT_BYTES + 1);
    assert_eq!(import_text(&huge, Vendor::Other), Err(ImportError::TooLarge));
}

#[test]
fn rejects_broken_json() {
    assert_eq!(import_text("{ not json", Vendor::Other), Err(ImportError::MalformedFile));
}

#[test]
fn surrounding_whitespace_and_a_byte_order_mark_are_ignored() {
    let link = format!("\u{feff}  otpauth://totp/a?secret={SECRET_B32} \n");
    assert_eq!(import(&link).accounts.len(), 1);
}

mod plain_links {
    use super::*;

    #[test]
    fn a_single_link_takes_the_vendor_the_user_picked() {
        let batch = import_text(&format!("otpauth://totp/Acme:a@b?secret={SECRET_B32}&issuer=Acme"), Vendor::Authy).unwrap();
        assert_eq!(first(&batch).vendor, Vendor::Authy);
        assert_eq!(first(&batch).account.secret, SECRET_BYTES);
    }

    #[test]
    fn a_list_of_links_one_per_line_imports_each_and_counts_the_rest() {
        let list = format!(
            "otpauth://totp/One?secret={SECRET_B32}\n\notpauth://totp/Two?secret={SECRET_B32}A\nnot a link\notpauth://hotp/Three?secret={SECRET_B32}&counter=1"
        );
        let batch = import(&list);
        assert_eq!(batch.accounts.len(), 2);
        assert_eq!(batch.skipped_unsupported, 2);
    }
}

mod aegis_exports {
    use super::*;

    fn export(entries: &str) -> String {
        format!(r#"{{"version":1,"header":{{"slots":null,"params":null}},"db":{{"version":3,"entries":[{entries}],"groups":[]}}}}"#)
    }

    fn entry(kind: &str) -> String {
        format!(
            r#"{{"type":"{kind}","uuid":"1","name":"alice@example.com","issuer":"GitHub","note":"","favorite":false,"icon":null,"info":{{"secret":"{SECRET_B32}","algo":"SHA256","digits":8,"period":60}},"groups":[]}}"#
        )
    }

    #[test]
    fn imports_the_secret_timing_and_labels_tagged_as_aegis() {
        let batch = import(&export(&entry("totp")));
        let account = &first(&batch).account;
        assert_eq!((account.issuer.as_str(), account.name.as_str()), ("GitHub", "alice@example.com"));
        assert_eq!((account.digits, account.period_seconds), (8, 60));
        assert_eq!(first(&batch).vendor, Vendor::Aegis);
    }

    #[test]
    fn counter_based_and_other_entry_types_are_skipped_not_fatal() {
        let batch = import(&export(&format!("{},{},{}", entry("totp"), entry("hotp"), entry("steam"))));
        assert_eq!((batch.accounts.len(), batch.skipped_unsupported), (1, 2));
    }

    #[test]
    fn an_encrypted_export_asks_for_an_unencrypted_one() {
        let encrypted = r#"{"version":1,"header":{"slots":[],"params":{}},"db":"c2VjcmV0"}"#;
        assert_eq!(import_text(encrypted, Vendor::Other), Err(ImportError::EncryptedExport("Aegis")));
    }
}

mod two_fas_backups {
    use super::*;

    fn backup(service: &str) -> String {
        format!(r#"{{"services":[{service}],"schemaVersion":4}}"#)
    }

    fn service(token_type: &str) -> String {
        format!(
            r#"{{"name":"GitLab","secret":"{SECRET_B32}","otp":{{"account":"foo@bar.com","counter":0,"period":30,"algorithm":"SHA1","issuer":"GitLab","tokenType":"{token_type}","digits":6,"source":"link"}}}}"#
        )
    }

    #[test]
    fn imports_labels_and_secret_tagged_as_2fas() {
        let batch = import(&backup(&service("TOTP")));
        let account = &first(&batch).account;
        assert_eq!((account.issuer.as_str(), account.name.as_str()), ("GitLab", "foo@bar.com"));
        assert_eq!(account.secret, SECRET_BYTES);
        assert_eq!(first(&batch).vendor, Vendor::TwoFas);
    }

    #[test]
    fn a_service_without_otp_details_falls_back_to_its_name() {
        let minimal = format!(r#"{{"services":[{{"name":"Acme","secret":"{SECRET_B32}"}}]}}"#);
        let account = import(&minimal).accounts.remove(0).account;
        assert_eq!((account.issuer.as_str(), account.name.as_str()), ("Acme", "Acme"));
    }

    #[test]
    fn non_time_based_tokens_are_skipped() {
        let batch = import(&backup(&format!("{},{}", service("TOTP"), service("HOTP"))));
        assert_eq!((batch.accounts.len(), batch.skipped_unsupported), (1, 1));
    }

    #[test]
    fn a_password_protected_backup_asks_for_an_unprotected_one() {
        let protected = r#"{"servicesEncrypted":"abc:def:ghi","schemaVersion":4}"#;
        assert_eq!(import_text(protected, Vendor::Other), Err(ImportError::EncryptedExport("2FAS")));
    }
}

mod bitwarden_exports {
    use super::*;

    fn export(items: &str) -> String {
        format!(r#"{{"encrypted":false,"folders":[],"items":[{items}]}}"#)
    }

    fn login(name: &str, totp: &str) -> String {
        format!(r#"{{"type":1,"name":"{name}","login":{{"username":"me@example.com","password":"x","totp":"{totp}"}}}}"#)
    }

    #[test]
    fn imports_an_otpauth_value_and_uses_the_item_name_when_it_has_no_issuer() {
        let batch = import(&export(&login("My Bank", &format!("otpauth://totp/me@example.com?secret={SECRET_B32}"))));
        assert_eq!((first(&batch).account.issuer.as_str(), first(&batch).account.name.as_str()), ("My Bank", "me@example.com"));
        assert_eq!(first(&batch).vendor, Vendor::Bitwarden);
    }

    #[test]
    fn imports_a_bare_secret_using_the_item_name_and_username() {
        let batch = import(&export(&login("GitHub", SECRET_B32)));
        assert_eq!((first(&batch).account.issuer.as_str(), first(&batch).account.name.as_str()), ("GitHub", "me@example.com"));
    }

    #[test]
    fn logins_without_a_code_are_ignored_and_steam_is_skipped() {
        let plain = r#"{"type":1,"name":"Forum","login":{"username":"u","password":"p","totp":null}}"#;
        let steam = login("Steam", "steam://ABCDEFGH");
        let batch = import(&export(&format!("{plain},{steam},{}", login("GitHub", SECRET_B32))));
        assert_eq!((batch.accounts.len(), batch.skipped_unsupported), (1, 1));
    }

    #[test]
    fn an_encrypted_export_asks_for_an_unencrypted_one() {
        let encrypted = r#"{"encrypted":true,"encKeyValidation_DO_NOT_EDIT":"2.x","data":"2.y"}"#;
        assert_eq!(import_text(encrypted, Vendor::Other), Err(ImportError::EncryptedExport("Bitwarden")));
    }

    #[test]
    fn imports_the_csv_export_including_quoted_fields() {
        let csv = format!(
            "folder,favorite,type,name,notes,fields,reprompt,login_uri,login_username,login_password,login_totp\n,0,login,\"Acme, Inc\",,,0,https://acme.test,bob@acme.test,pw,{SECRET_B32}\n,0,login,Forum,,,0,,u,p,\n"
        );
        let batch = import(&csv);
        assert_eq!(batch.accounts.len(), 1);
        assert_eq!((first(&batch).account.issuer.as_str(), first(&batch).account.name.as_str()), ("Acme, Inc", "bob@acme.test"));
        assert_eq!(first(&batch).vendor, Vendor::Bitwarden);
    }
}
