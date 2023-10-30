# BUAA Boya course automation tool
> It's just a learning project, invaded and deleted
## Prepare
1. Install Chromium
2. Install Chrome driver (The version matches Chromium)
## Install from source (Need Rust toolchains)
```bash
git clone https://githubfast.com/fangtaluosi/auto-boya.git
cd auto-boya
cargo build --release
```
## Usage
1. `auto-boya init` to configure the chromium, your account and your password
2. `auto-boya run`, and it should print:
    ```
    [info] Read config
    [info] Set success
    [info] Check Chrome ...
    [info] Check driver ...
    [info] Navigate to https://sso.buaa.edu.cn/login
    [info] Input account and password
    [info] Login
    [info] Redirect to https://bykc.buaa.edu.cn
    [info] Get course list
    ... a course table
    ? Waiting for input index...  ›
    ```
3. Input index and press enter

## ToDo
- [x] Login
- [x] Choose avaliable course
- [ ] Choose preview course
