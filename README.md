# axum todo demo

## 简介

### rust框架

[axum](https://github.com/tokio-rs/axum)

[tokio-postgres](https://crates.io/crates/tokio-postgres)

### 压测项目

[axum-todo-k6-test](https://github.com/liuquanhao/axum-todo-k6-test)

## 项目说明

### 创建数据库/表

```bash
$ psql
postgres=# CREATE USER todouser WITH ENCRYPTED PASSWORD 'todopassword';
postgres=# CREATE DATABASE todos;
postgres=# GRANT ALL PRIVILEGES ON DATABASE todos to todouser;
postgres=# \c todos
postgres=# CREATE TABLE todos (id UUID PRIMARY KEY NOT NULL, text HAR(255) NOT NULL DEFAULT '', completed BOOLEAN NOT NULL DEFAULT false);
```

### 运行项目

```bash
$ POSTGRESQL_URL="postgres://todouser:todopassword@127.0.0.1:5432/s" cargo run
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
