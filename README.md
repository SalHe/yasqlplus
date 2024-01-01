# YaSQL+

[![build](https://github.com/SalHe/yasqlplus/actions/workflows/build.yml/badge.svg)](https://github.com/SalHe/yasqlplus/actions/workflows/build.yml)

YaSQL+ is an alternative for yasql.

Yasql is an official CLI client for [YashanDB](https://www.yashandb.com/) designed and developed by [SICS(Shenzhen Institute of Computing Sciences)](https://www.sics.ac.cn/).

YaSQL+ has some features for better experience, but not shipped with latest YashanDB feature compared with yasql.

**This project is still in developing.**

## Features

- [x] Syntax highlighting
- [x] Command history/search/completion
- [x] [Vim key bindings](https://github.com/kkawakam/rustyline?tab=readme-ov-file#vi-command-mode)
- [x] Display wide content in page program (Such as more/less)
- [x] Multiline input
- [ ] Smart completion for Keywords/Table/View/Column
- [ ] Alias command

![yasqlplus](./docs/images/yasqlplus.gif)

## Installation

You could install YaSQL+ via source.

```shell
git clone https://github.com/SalHe/yasqlplus.git
cd yasqlplus
cargo install --path .
```

Or download [prebuilt binaries](https://github.com/SalHe/yasqlplus/releases).

Currently YaSQL+ supports only linux because c driver is provided by [YashanDB](https://download.yashandb.com/download) for linux only.

## Usage

### Use YaSQL+ as an interactive shell.

```shell
➜  ~ yasqlplus
SQL > conn sys/Cod-2022
Connected!
sys@127.0.0.1:1688 > -- prompt will show connection
sys@127.0.0.1:1688 > select * from v$instance;
╭────────┬────────────────────────────────────────────┬────────────────────────────┬───────────────────┬─────────────────────────────────┬─────────────────┬───────────────┬──────────┬───────────────┬───────────╮
│ STATUS │ VERSION                                    │ STARTUP_TIME               │ HOST_NAME         │ DATA_HOME                       │ INSTANCE_NUMBER │ INSTANCE_NAME │ PARALLEL │ INSTANCE_ROLE │ IN_REFORM │
├────────┼────────────────────────────────────────────┼────────────────────────────┼───────────────────┼─────────────────────────────────┼─────────────────┼───────────────┼──────────┼───────────────┼───────────┤
│ OPEN   │ Personal Edition Release 23.1.1.100 x86_64 │ 2023-12-15 21:30:11.008174 │ ----------------- │ /home/yasha/yashandb/yasdb_data │ 1               │ yasdb         │ false    │ MASTER_ROLE   │ NO        │
╰────────┴────────────────────────────────────────────┴────────────────────────────┴───────────────────┴─────────────────────────────────┴─────────────────┴───────────────┴──────────┴───────────────┴───────────╯
1 row(s) fetched
sys@127.0.0.1:1688 > desc v$instance;
╭──────────────┬─────────────────┬──────┬───────────┬──────────┬───────────┬───────┬───────────┬───────────────────╮
│ display_size │ name            │ size │ type_     │ nullable │ precision │ scale │ char_size │ display_char_size │
├──────────────┼─────────────────┼──────┼───────────┼──────────┼───────────┼───────┼───────────┼───────────────────┤
│ 12           │ STATUS          │ 12   │ VARCHAR   │ true     │ 0         │ 0     │ 12        │ 12                │
│ 64           │ VERSION         │ 64   │ VARCHAR   │ true     │ 0         │ 0     │ 64        │ 64                │
│ 64           │ STARTUP_TIME    │ 8    │ TIMESTAMP │ true     │ 38        │ 128   │ 8         │ 64                │
│ 256          │ HOST_NAME       │ 256  │ VARCHAR   │ true     │ 0         │ 0     │ 256       │ 256               │
│ 256          │ DATA_HOME       │ 256  │ VARCHAR   │ true     │ 0         │ 0     │ 256       │ 256               │
│ 11           │ INSTANCE_NUMBER │ 4    │ INTEGER   │ true     │ 38        │ 128   │ 4         │ 11                │
│ 64           │ INSTANCE_NAME   │ 64   │ VARCHAR   │ true     │ 0         │ 0     │ 64        │ 64                │
│ 5            │ PARALLEL        │ 1    │ BOOL      │ true     │ 38        │ 128   │ 1         │ 5                 │
│ 64           │ INSTANCE_ROLE   │ 64   │ VARCHAR   │ true     │ 0         │ 0     │ 64        │ 64                │
│ 8            │ IN_REFORM       │ 8    │ VARCHAR   │ true     │ 0         │ 0     │ 8         │ 8                 │
╰──────────────┴─────────────────┴──────┴───────────┴──────────┴───────────┴───────┴───────────┴───────────────────╯
sys@127.0.0.1:1688 > -- Ctrl + C to interrupt input
sys@127.0.0.1:1688 > -- Ctrl + D to exit yasqlplus
sys@127.0.0.1:1688 >
➜  ~
```

### Tricks

- To save result to file, you could redirect stdout to a file, like `echo 'SELECT * FROM dba_Tables limit 1;' | yasqlplus sys/Cod-2022 > output`
- To disable show wide content in `less` you could pipe out to `cat` like this `echo 'SELECT * FROM dba_Tables limit 1;' | yasqlplus sys/Cod-2022 | cat`.


## Limitations

Some features are not supported now (No related public API found), such as:

- Logon response server version (Output `YashanDB Server Personal Edition Release 23.1.1.100 x86_64 - X86 64bit Linux` when connected)
- Server output response (`set serverout off`)
