use anyhow::bail;
use headless_chrome::{Browser, LaunchOptionsBuilder};

fn translate_status(status: &str) -> anyhow::Result<String> {
    match status {
        "レギュラーステージ" => Ok("Regular Stage".to_string()),
        "ブロンドステージ" => Ok("Bronze Stage".to_string()),
        "シルバーステージ" => Ok("Silver Stage".to_string()),
        "ゴールドステージ" => Ok("Gold Stage".to_string()),
        "プラチナステージ" => Ok("Platina Stage".to_string()),
        _ => bail!("Status translation failed."),
    }
}

pub fn retrieve_status(email: &str, password: &str) -> anyhow::Result<String> {
    let launch_options = LaunchOptionsBuilder::default().sandbox(false).build()?;
    let browser = Browser::new(launch_options)?;

    let tab = browser.new_tab()?;
    tab.enable_stealth_mode()?;

    tab.navigate_to("https://coco-web.jp/users/login")?;

    tab.wait_for_element("input[name=email]")?.click()?;
    tab.type_str(email)?;
    tab.wait_for_element("input[name=password]")?.click()?;
    tab.type_str(password)?;

    tab.wait_for_element("button[type=submit]")?.click()?;

    // 利用規約の更新について
    if let Ok(el) = tab.wait_for_element("button[type=submit]") {
        el.click()?;
    }

    let status = tab.wait_for_element(".stage_area")?.get_inner_text()?;

    translate_status(&status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn can_get_status() {
        let email = todo!("Input test email");
        let password = todo!("Input test password");
        let status = retrieve_status(email, password).unwrap();
        assert_eq!("Regular Stage".to_string(), status);
    }
}
