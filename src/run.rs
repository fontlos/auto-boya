use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input};
use nom::{
    bytes::complete::{tag, take_until},
    combinator::{map, rest},
    multi::many0,
    sequence::{separated_pair, terminated, tuple},
    IResult,
};
use tabled::{
    settings::{
        object::{Rows, Segment},
        Alignment, Modify, Style, Width,
    },
    Table, Tabled,
};
use thirtyfour::prelude::*;
use tokio::time::{sleep, Duration};

use std::process::{Command, Stdio};

use crate::config::{set_config, Setting};

const URL: &str =
    "https://sso.buaa.edu.cn/login?TARGET=http%3A%2F%2Fbykc.buaa.edu.cn%2Fsscv%2FcasLogin";

pub async fn run(set: Setting) -> WebDriverResult<()> {
    let cfg = set_config(set);

    match Command::new(&cfg.chrome_driver)
        .arg(&cfg.driver_port)
        .stdout(Stdio::null())
        .spawn()
    {
        Ok(_) => {
            println!("{} {}", "[info]".green().bold(), "Start Chrome driver");
        }
        Err(e) => {
            println!(
                "{} Can't start Chrome driver because {}",
                "[error]".red().bold(),
                e
            )
        }
    }

    let mut caps = DesiredCapabilities::chrome();
    caps.add_chrome_arg("headless")?;
    // 防止headless检测
    // caps.add_chrome_arg("--window-size=1920,1080")?;
    caps.add_chrome_arg(r#"user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36"#)?;
    caps.add_chrome_option("excludeSwitches", ["enable-automation"])?;
    // caps.add_chrome_option("useAutomationExtension", false)?;
    println!("{} {}", "[info]".green().bold(), "Check Chrome ...");
    match caps.set_binary(&cfg.chrome_binary) {
        Ok(_) => (),
        Err(_) => {
            println!(
                "{} {}",
                "[Error]".red().bold(),
                "Failed to find chrome binary"
            );
            return Ok(());
        }
    };
    println!("{} {}", "[info]".green().bold(), "Check driver ...");
    let driver = match WebDriver::new(&format!("http://localhost:{}", cfg.driver_port), caps).await
    {
        Ok(d) => d,
        Err(_) => {
            println!(
                "{} {}",
                "[Error]".red().bold(),
                "Failed to connect chrome driver"
            );
            return Ok(());
        }
    };

    // Navigate to https://sso.buaa.edu.cn/login.
    driver.goto(URL).await?;
    let login_iframe = driver.find(By::Id("loginIframe")).await?;
    println!(
        "{} {}",
        "[info]".green().bold(),
        "Navigate to https://sso.buaa.edu.cn/login"
    );

    // Switch to the iframe so that we can find the elements in it.
    login_iframe.enter_frame().await?;

    // Input account
    let account_input = driver.find(By::Id("unPassword")).await?;
    account_input.send_keys(cfg.account).await?;

    // Input password
    let password_input = driver.find(By::Id("pwPassword")).await?;
    password_input.send_keys(cfg.password).await?;
    println!(
        "{} {}",
        "[info]".green().bold(),
        "Input account and password"
    );

    // Click the login button.
    let login_button = driver
        .query(By::Css("input[type='button'].submit-btn"))
        .first()
        .await?;
    login_button.click().await?;
    println!("{} {}", "[info]".green().bold(), "Login");

    match driver.goto("https://bykc.buaa.edu.cn/system/home").await {
        Ok(_) => println!(
            "{} {}",
            "[info]".green().bold(),
            "Redirect to https://bykc.buaa.edu.cn"
        ),
        Err(_) => {
            println!("{} {}", "[error]".green().bold(), "Redirect failed");
            return Ok(());
        }
    };

    driver
        .query(By::Css(
            "div[ng-click='menu.showSubmenu=!menu.showSubmenu'].nav-main",
        ))
        .first()
        .await?
        .click()
        .await?;
    driver
        .query(By::Css("li[ui-sref='system.course-select'].ng-scope"))
        .first()
        .await?
        .click()
        .await?;

    let course_list = driver.query(By::Css("tr.ng-scope")).all().await?;
    println!("{} {}", "[info]".green().bold(), "Get course list");

    let mut courses = Vec::new();

    for (index, c) in course_list.iter().enumerate() {
        let ct = c.text().await?;
        match parse_course(&ct, index) {
            Ok((_, opt_course)) => match opt_course {
                Some(course) => courses.push(course),
                None => continue,
            },
            Err(_) => {
                continue;
            }
        };
    }
    let mut table = Table::new(courses);
    table
        .with(Style::rounded())
        .with(Modify::new(Rows::new(3..=3)).with(Width::wrap(40).keep_words()))
        .with(Modify::new(Segment::all()).with(Alignment::center()));
    println!("{table}");

    let index: usize = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Waiting for input index... ")
        .validate_with(|n: &usize| {
            if *n < course_list.len() {
                Ok(())
            } else {
                Err("Invalid input. It is out of range")
            }
        })
        .interact_text()
        .unwrap();

    let chosen_course = match course_list.get(index) {
        Some(c) => c,
        None => {
            println!(
                "{} {}",
                "[error]".red().bold(),
                "Invalid input. It is out of range"
            );
            return Ok(());
        }
    };

    let choose_button = chosen_course
        .query(By::Css("a.text-success.block.ng-scope"))
        .first()
        .await?;
    match choose_button.click().await {
        Ok(_) => (),
        Err(_) => println!("{} {}", "[error]".green().bold(), "Choose failed"),
    };

    let ok_button = driver
        .query(By::Css("button.btn.btn-primary.ng-binding"))
        .first()
        .await?;
    match ok_button.click().await {
        Ok(_) => println!("{} {}", "[info]".green().bold(), "Choose success"),
        Err(_) => println!("{} {}", "[error]".green().bold(), "Choose failed"),
    };

    sleep(Duration::from_secs(1)).await;

    // Always explicitly close the browser.
    driver.quit().await?;
    println!("{} {}", "[info]".green().bold(), "Program quit success");

    Ok(())
}
/// Available
fn a(i: &str) -> IResult<&str, bool> {
    use nom::character::complete::u16;
    map(separated_pair(u16, tag("/"), u16), |x| x.0 < x.1)(i)
}
/// Separated by :
fn s(i: &str) -> IResult<&str, &str> {
    map(tuple((take_until("："), tag("："), rest)), |x| x.2)(i)
}
/// Course type
fn t(i: &str) -> IResult<&str, (&str, &str)> {
    map(
        tuple((take_until(" 博雅课程-"), tag(" 博雅课程-"), rest)),
        |x| (x.0, x.2),
    )(i)
}

#[derive(Tabled)]
struct Course {
    /// course index
    #[tabled(rename = "Index")]
    index: usize,
    /// course state
    #[tabled(rename = "State")]
    state: String,
    /// course type
    #[tabled(rename = "Type")]
    t: String,
    /// course name
    #[tabled(rename = "Name")]
    name: String,
    /// course address
    #[tabled(rename = "Address")]
    ad: String,
    /// course time
    #[tabled(rename = "Start time")]
    st: String,
    /// course choose time
    #[tabled(rename = "Choose time")]
    ct: String,
}

// fn parse_course2<'a>(i: &'a str, index: &'a str) -> IResult<&'a str, Option<Vec<&'a str>>> {
//     map(many0(terminated(take_until("\n"), tag("\n"))), |x| {
//         let (_, (n,t)) = t(x[1]).unwrap();
//         if (x[0] == "预告" || x[0] == "可选") && a(x[16]).unwrap().1 {
//             Some(vec![
//                 index,
//                 x[0],
//                 t,
//                 n,
//                 s(x[2]).unwrap().1,
//                 s(x[5]).unwrap().1,
//                 s(x[12]).unwrap().1,
//             ])
//         } else {
//             None
//         }
//     })(i)
// }

fn parse_course(i: &str, index: usize) -> IResult<&str, Option<Course>> {
    map(many0(terminated(take_until("\n"), tag("\n"))), |x| {
        let (_, (n, t)) = t(x[1]).unwrap();
        if (x[0] == "预告" || x[0] == "可选") && a(x[16]).unwrap().1 {
            Some(Course {
                index,
                state: x[0].to_string(),
                t: t.to_string(),
                name: n.to_string(),
                ad: s(x[2]).unwrap().1.to_string(),
                st: s(x[5]).unwrap().1.to_string(),
                ct: s(x[12]).unwrap().1.to_string(),
            })
        } else {
            None
        }
    })(i)
}

#[test]
fn test_term() {
    fn p(i: &str) -> IResult<&str, Option<Vec<&str>>> {
        map(many0(terminated(take_until("\n"), tag("\n"))), |x| {
            let (_, (n, t)) = t(x[1]).unwrap();
            if x[0] == "可选" && a(x[16]).unwrap().1 {
                Some(vec![
                    x[0],
                    n,
                    t,
                    s(x[2]).unwrap().1,
                    s(x[5]).unwrap().1,
                    s(x[12]).unwrap().1,
                ])
            } else {
                None
            }
        })(i)
    }
    let strs1 = "可选\n生命在于睡好 博雅课程-安全健康\n地点：学院路新主楼第二报告厅\n教师：唐林\n学院：学生工作部（处）/武装部\n开始：2023-10-26 14:00\n结束：2023-10-26 15:40\n校区：全部校区\n学院：全部学院\n年级：全部年级\n人群：全部人群\n选课方式：直接选课\n选课开始：2023-10-24 11:00\n选课结束：2023-10-25 18:00\n退选截止：2023-10-25 18:00\n无作业\n120/120\n详细介绍";
    let strs2 = "可选\n中法大讲堂名师讲坛系列讲座第二期 博雅课程-德育\n地点：北京航空航天大学学院路校区二号教学楼305教室\n教师：孙大坤\n学院：中法工程师学院\n开始：2023-10-25 14:30\n结束：2023-10-25 16:30\n校区：全部校区\n学院：全部学院\n年级：全部年级\n人群：全部人群\n选课方式：直接选课\n选课开始：2023-10-24 10:00\n选课结束：2023-10-25 10:00\n退选截止：2023-10-25 13:00\n无作业\n97/100\n详细介绍";
    let (_r, v) = p(strs1).unwrap();
    println!("{:?}", v);
    let (_r, v) = p(strs2).unwrap();
    let mut b = tabled::builder::Builder::new();
    let v = v.unwrap();
    b.push_record(v);
    b.set_header([
        "state",
        "name",
        "type",
        "address",
        "start time",
        "choosing start time",
    ]);
    let mut t = b.build();
    // let mut t = b.build();
    t.with(tabled::settings::Style::rounded());
    // let t = t.to_string();
    println!("{}", t);
}
