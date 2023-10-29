use colored::Colorize;
use nom::{
    bytes::complete::{tag, take_until},
    combinator::{map, rest},
    sequence::{separated_pair, terminated, tuple},
    IResult,
    multi::many0,
};
use tabled::{settings::Style, Table, Tabled};
use thirtyfour::prelude::*;
use tokio::time::{Duration, sleep};

const ACCOUNT: &str = "23379025";
const PASSWORD: &str = "52molimuyan1314";
const BINARY: &str = "D:\\Programming\\Tool\\chromium\\chrome.exe";
const URL: &str = "https://sso.buaa.edu.cn/login?TARGET=http%3A%2F%2Fbykc.buaa.edu.cn%2Fsscv%2FcasLogin";

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let mut caps = DesiredCapabilities::chrome();
    caps.add_chrome_arg("headless")?;
    // 航子你的无头检测真垃圾, 加个agent就能过
    // caps.add_chrome_arg("--window-size=1920,1080")?;
    caps.add_chrome_arg(r#"user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36"#)?;
    caps.add_chrome_option("excludeSwitches", ["enable-automation"])?;
    caps.add_chrome_option("useAutomationExtension", false)?;
    match caps.set_binary(BINARY){
        Ok(_)=>(),
        Err(_)=>{
            println!("{} {}","[Error]".red().bold(),"Failed to find chrome binary");
            return Ok(());
        }
    };
    let driver = match WebDriver::new("http://localhost:9515", caps).await{
        Ok(d)=>d,
        Err(_)=>{
            println!("{} {}","[Error]".red().bold(),"Failed to connect chrome driver");
            return Ok(());
        }
    };

    // Navigate to https://sso.buaa.edu.cn/login.
    driver.goto(URL).await?;
    let login_iframe = driver.find(By::Id("loginIframe")).await?;
    println!("{} {}","[info]".green().bold(),"Navigate to https://sso.buaa.edu.cn/login");

    // Switch to the iframe so that we can find the elements in it.
    login_iframe.enter_frame().await?;

    // Input account
    let account_input = driver.find(By::Id("unPassword")).await?;
    account_input.send_keys(ACCOUNT).await?;

    // Input password
    let password_input = driver.find(By::Id("pwPassword")).await?;
    password_input.send_keys(PASSWORD).await?;
    println!("{} {}","[info]".green().bold(),"Input account and password");

    // Click the login button.
    let login_button = driver.query(By::Css("input[type='button'].submit-btn")).first().await?;
    login_button.click().await?;
    println!("{} {}","[info]".green().bold(),"Login");

    match driver.goto("https://bykc.buaa.edu.cn/system/home").await{
        Ok(_)=>println!("{} {}","[info]".green().bold(),"Redirect to https://bykc.buaa.edu.cn"),
        Err(_)=>{
            println!("{} {}","[error]".green().bold(),"Redirect failed");
            return Ok(())
        }
    };

    driver.query(By::Css("div[ng-click='menu.showSubmenu=!menu.showSubmenu'].nav-main"))
        .first()
        .await?
        .click()
        .await?;
    driver.query(By::Css("li[ui-sref='system.course-select'].ng-scope"))
        .first()
        .await?
        .click()
        .await?;

    let course_list = driver.query(By::Css("tr.ng-scope")).all().await?;
    println!("{} {}","[info]".green().bold(),"Get course list");

    let mut courses = Vec::new();

    for (index, c) in course_list.iter().enumerate() {
        let ct = c.text().await?;
        match parse_course(&ct, index){
            Ok((_, opt_course))=>{
                match opt_course{
                    Some(course)=>courses.push(course),
                    None=>continue,
                }
            },
            Err(_)=>{
                continue;
            }
        };
    }

    let mut table = Table::new(courses);
    table.with(Style::rounded());
    println!("{table}");

    print!("{} {}","[info]".green().bold(),"Waiting for input index...");

    let mut choose = String::new();

    std::io::stdin().read_line(&mut choose).unwrap();
    let index = match choose.trim().parse::<usize>(){
        Ok(c)=>c,
        Err(_)=>{
            println!("{} {}","[error]".red().bold(),"Invalid input. It is not a number");
            return Ok(())
        }
    };

    let chosen_course =match course_list.get(index){
        Some(c)=>c,
        None=>{
            println!("{} {}","[error]".red().bold(),"Invalid input. It is out of range");
            return Ok(())
        }
    };

    let choose_button = chosen_course.query(By::Css("a.text-success.block.ng-scope")).first().await?;
    match choose_button.click().await{
        Ok(_)=>(),
        Err(_)=> println!("{} {}","[error]".green().bold(),"Choose failed")
    };

    let ok_button = driver.query(By::Css("button.btn.btn-primary.ng-binding")).first().await?;
    match ok_button.click().await{
        Ok(_)=>println!("{} {}","[info]".green().bold(),"Choose success"),
        Err(_)=> println!("{} {}","[error]".green().bold(),"Choose failed")
    };

    sleep(Duration::from_secs(2)).await;

    // Always explicitly close the browser.
    driver.quit().await?;
    println!("{} {}","[info]".green().bold(),"Program quit success");

    Ok(())
}

fn s(i: &str)->IResult<&str, bool>{
    use nom::character::complete::u16;
    map(
        separated_pair(u16, tag("/"), u16),
        |x|x.0<x.1
    )(i)
}

fn t(i: &str)->IResult<&str, &str>{
    map(
        tuple((
            take_until("："),
            tag("："),
            rest,
        )),
        |x|x.2
    )(i)
}

#[derive(Tabled)]
struct Course{
    index: usize,
    /// course name
    name: String,
    /// course address
    address: String,
    /// course time
    start_time: String,
    /// course choose time
    choose_time: String
}

fn parse_course(i: &str, index: usize)->IResult<&str,Option<Course>>{
    map(
        many0(
            terminated(take_until("\n"), tag("\n"))
        ),
        |x|{
            if (x[0]=="可选"||x[0]=="预告")&&s(x[16]).unwrap().1{
                Some(Course{
                    index,
                    name: x[1].to_string(),
                    address: t(x[2]).unwrap().1.to_string(),
                    start_time:t(x[5]).unwrap().1.to_string(),
                    choose_time: t(x[12]).unwrap().1.to_string()
                })
            }else{
                None
            }
        }
    )(i)
}

#[test]
fn test_term(){
    fn p(i: &str)->IResult<&str,Option<Vec<&str>>>{
        map(
            many0(
                terminated(take_until("\n"), tag("\n"))
            ),
            |x|{
                if x[0]=="可选"&&s(x[16]).unwrap().1{
                    Some(vec![x[1], t(x[2]).unwrap().1, t(x[5]).unwrap().1, t(x[12]).unwrap().1])
                }else{
                    None
                }
            }
        )(i)
    }
    let strs1 = "可选\n生命在于睡好 博雅课程-安全健康\n地点：学院路新主楼第二报告厅\n教师：唐林\n学院：学生工作部（处）/武装部\n开始：2023-10-26 14:00\n结束：2023-10-26 15:40\n校区：全部校区\n学院：全部学院\n年级：全部年级\n人群：全部人群\n选课方式：直接选课\n选课开始：2023-10-24 11:00\n选课结束：2023-10-25 18:00\n退选截止：2023-10-25 18:00\n无作业\n120/120\n详细介绍";
    let strs2 = "可选\n中法大讲堂名师讲坛系列讲座第二期 博雅课程-德育\n地点：北京航空航天大学学院路校区二号教学楼305教室\n教师：孙大坤\n学院：中法工程师学院\n开始：2023-10-25 14:30\n结束：2023-10-25 16:30\n校区：全部校区\n学院：全部学院\n年级：全部年级\n人群：全部人群\n选课方式：直接选课\n选课开始：2023-10-24 10:00\n选课结束：2023-10-25 10:00\n退选截止：2023-10-25 13:00\n无作业\n97/100\n详细介绍";
    let (_r,v) = p(strs1).unwrap();
    println!("{:?}",v);
    let (_r,v) = p(strs2).unwrap();
    let mut b = tabled::builder::Builder::new();
    let v = v.unwrap();
    b.push_record(v);
    b.set_header(["name","address","start time","choosing start time"]);
    let mut t = b.index().transpose().build();
    // let mut t = b.build();
    t.with(tabled::settings::Style::rounded());
    // let t = t.to_string();
    println!("{}",t);
}
