use anyhow::bail;
use headless_chrome::{Browser, LaunchOptionsBuilder};

fn translate_status(status: &str) -> anyhow::Result<String> {
    match status {
        "メンバー" => Ok("Member".to_string()),
        "シルバー" => Ok("Silver".to_string()),
        "ゴールド" => Ok("Gold".to_string()),
        _ => bail!("Status translation failed."),
    }
}

pub fn retrieve_status(email: &str, password: &str) -> anyhow::Result<String> {
    let launch_options = LaunchOptionsBuilder::default().sandbox(false).build()?;
    let browser = Browser::new(launch_options)?;

    let tab = browser.new_tab()?;
    tab.enable_stealth_mode()?;

    tab.navigate_to("https://www.hotespa.net/dormyinn/")?;

    tab.wait_for_element(".logent > a")?.click()?;

    tab.wait_for_element("input[name=mailAddress]")?.click()?;
    tab.type_str(email)?;
    tab.wait_for_element("input[name=password]")?.click()?;
    tab.type_str(password)?;

    tab.wait_for_element(".formSubmit")?.click()?;

    if let Ok(el) = tab.wait_for_element("#warnOkButton") {
        el.click()?;
    }

    tab.wait_for_element("a[href*=mypage]")?;

    tab.navigate_to("https://www.kyoritsumembers.com/secure/mypage/member")?;

    if let Ok(el) = tab.wait_for_element("#warnOkButton") {
        el.click()?;
    }

    let status = tab.wait_for_element(".serviceType")?.get_inner_text()?;
    translate_status(&status)
}
