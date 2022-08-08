# Http_Mock_Server

本程序是一个模拟接口响应的独立服务器。本程序根据请求动态生成响应数据，并且可以通过ui界面动态调整。若想用编写代码来作单元测试,集成测试，Java语言可以使用[mockserver](https://github.com/mock-server/mockserver)，Rust则可以使用[httpmock](https://github.com/alexliesenfeld/httpmock)

## 开发背景

项目开发中有需要用到模拟服务器来模拟接口数据，之前使用过Java的[mockserver](https://github.com/mock-server/mockserver)和[httpmock](https://github.com/alexliesenfeld/httpmock)的独立服务器模式，但是有几个缺点：

- 无法根据请求动态生成响应数据。

- 匹配规则多设置麻烦。且没有配置UI界面。靠调接口配置。

- 当请求不匹配时，服务器直接返回404，没有一些不匹配的信息。且验证麻烦。

- 服务器信息展示界面不清晰。

- 性能不好。上述两个模拟服务器都是遍历所有规则，直到找到一个匹配的。

- 在就是Java的缺点了，占用内存高。

在github上查了一段时间，并没有发现符合我的需求的。所以决定自己来开发一个。之前打算用Java。但是考虑这个定位是个辅助小工具，就得有个小工具的样子。正好之前有空的时候把rust的helloworld指导手册看完了。就决定用rust来边学边开发了。

## 应用场景

![](C:\Users\黄金\AppData\Roaming\marktext\images\2022-08-03-17-02-30-image.png)

## 特点介绍

- 小，执行程序14Mb。运行内存占用20Mb。在应对4000的并发的时候，内存占用也才200多Mb。而Java虚拟机启动时就会占用200多了。

- 采用异步io。性能强劲，没有具体测过。但是在我笔记本上可轻松达到4000的并发。

- 规则配置采用正则表达式校验。

- 以路径为首要条件。不匹配时，返回哪些条件不匹配的信息。

- 采用[Jinja](https://docs.rs/minijinja/latest/minijinja/syntax/index.html)模板引擎。可以根据请求参数设置动态返回响应数据。

## 使用介绍

1. 配置页面
