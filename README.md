# YaSQL+

[![build](https://github.com/SalHe/yasqlplus/actions/workflows/build.yml/badge.svg)](https://github.com/SalHe/yasqlplus/actions/workflows/build.yml)

YaSQL+ is an alternative for yasql. 

Yasql is an offical CLI client for [YashanDB](https://www.yashandb.com/) designed and developed by [SICS(Shenzhen Institute of Computing Sciences)](https://www.sics.ac.cn/)

YaSQL+ has some features for better experience, but not shipped with latest YashanDB feature compared with yasql.

**This project is still in developing.**

## Features

- [x] Syntax highlighting
- [x] Command history/search/completion
- [x] [Vim key bindings](https://github.com/kkawakam/rustyline?tab=readme-ov-file#vi-command-mode)
- [x] Display wide content in page program (Such as more/less)
- [ ] Smart multiline input
- [ ] Smart completion for Keywords/Table/View/Column
- [ ] Alias command

## Limitations

Some features are not supported now (No related public API found), such as: 

- Logon response server version (Output `YashanDB Server Personal Edition Release 23.1.1.100 x86_64 - X86 64bit Linux` when connected)
- Server output response (`set serverout off`)
