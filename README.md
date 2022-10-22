# axum todo demo

## 简介

### rust框架

[axum](https://github.com/tokio-rs/axum)

[tokio-postgres](https://crates.io/crates/tokio-postgres)

### 数据库

postgresql-15

### 压测项目

[axum-todo-k6-test](https://github.com/liuquanhao/axum-todo-k6-test)

apache2-utils: ab 2.3

## 项目说明

### 创建数据库/表

```bash
$ psql
postgres=# CREATE USER todouser WITH ENCRYPTED PASSWORD 'todopassword';
postgres=# CREATE DATABASE todos;
postgres=# GRANT ALL PRIVILEGES ON DATABASE todos to todouser;
postgres=# \c todos;
postgres=# CREATE TABLE todos (id UUID PRIMARY KEY NOT NULL, text VARCHAR(255) NOT NULL DEFAULT '', completed BOOLEAN NOT NULL DEFAULT false);
postgres=# GRANT ALL ON todos TO todouser;
```

### 性能优化

```bash
$ ulimit -n
102400
$ cat /etc/sysctl.conf
net.ipv4.ip_local_port_range=1024 65535
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_timestamps = 1

$ cat /etc/postgresql/15/main/postgresql.conf
# 修改关键配置
max_connections = 2000
ssl = false
# 具体看sql，简单sql 4MB也够用
work_mem = 64MB
maintenance_work_mem = 256MB
wal_level = minimal
synchronous_commit = off
max_wal_senders = 0
# ssd用2.0或1.0
random_page_cost = 2.0
# 服务器可用内存
effective_cache_size = 1GB
```

### 运行项目

```bash
$ cargo build --release 
$ POSTGRESQL_URL="postgres://todouser:todopassword@127.0.0.1:5432/todos" ./target/release/axum-todo
```

### 接口

#### 获取todo列表

GET: /todos/

#### 获取某个todo

GET: /todos/:id

#### 创建todo

POST: /todos/

header: content-type:application/json

body: {"text": "todo test"}

#### 修改todo

PUT: /todos/:id

header: content-type:application/json

body: {"text": "todo test2"}

#### 删除todo

DELETE: /todos/:id
