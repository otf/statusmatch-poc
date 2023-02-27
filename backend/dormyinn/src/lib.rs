use headless_chrome::Browser;

pub fn retrieve_status(email: &str, password: &str) -> anyhow::Result<String> {
    let browser = Browser::default()?;

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

    tab.wait_for_element(".serviceType")?.get_inner_text()
}
