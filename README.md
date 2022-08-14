# Http Mock Server

本程序是一个模拟接口响应的独立服务器。本程序根据请求动态生成响应数据，并且可以通过ui界面动态调整。若想使用模拟测试库的话，Java语言可以使用[mockserver](https://github.com/mock-server/mockserver)，Rust则可以使用[httpmock](https://github.com/alexliesenfeld/httpmock)

## 开发动机

项目开发中有需要用到模拟服务器来模拟接口数据，之前使用过Java的[mockserver](https://github.com/mock-server/mockserver)和[httpmock](https://github.com/alexliesenfeld/httpmock)的独立服务器模式，但是有几个缺点：

- 无法根据请求动态生成响应数据。

- 匹配规则多设置麻烦。且没有配置UI界面。靠调接口配置。

- 当请求不匹配时，服务器直接返回404，没有一些不匹配的信息。且验证麻烦。

- 服务器信息展示界面不清晰。

![]()在github上查了一段时间，并没有发现符合我的需求。

## 应用场景

主要用于模拟接口数据、简单的接口管理、动态调整接口数据等场景,定位用户为测试、项目管理人员。

![draw.io_zp8l9BK2pQ.png](D:\1-code\http_mock_server\screenshots\draw.io_zp8l9BK2pQ.png)

## 特点介绍

- 小，执行程序15Mb。在应对4000的并发的时候，内存占用也才200多Mb。空闲情况下占用内存5M左右。

- 采用异步io。性能强劲。

- 规则配置采用正则表达式校验。

- ui配置界面，简单易用。

- 以路径为首要条件。不匹配时，返回条件不匹配的信息。

- 采用[Jinja](https://docs.rs/minijinja/latest/minijinja/syntax/index.html)模板引擎。可以根据请求参数设置动态返回响应数据。

## 使用介绍

1. 介绍
   - 配置管理界面
   
   ![](D:\1-code\http_mock_server\screenshots\httpmock_ui_FM6cbSVfAN.png)
   
   - 查看信息界面(web)[http://localhost:13001/_mock_info/]()


2. 配置


3. 日志


4. 注意事项
 
## TODO