# Concurrent HTTP/1.1 Server in C

A lightweight, multi-client HTTP/1.1 server built from scratch in C using POSIX sockets.
This project focuses on low-level networking, protocol parsing, and concurrent request handling.

---

## 🚀 Overview

This server implements core parts of the HTTP/1.1 protocol, including request parsing, routing, and response generation. It supports concurrent client handling using threads and demonstrates system-level programming concepts such as socket communication, dynamic memory management, and file I/O.

---

## ⚙️ Features

* **Concurrent Client Handling**

  * Multi-client support using `pthread` (thread-per-client model)

* **HTTP Request Parsing**

  * Parses request line (method, path)
  * Extracts headers and `Content-Length`
  * Handles CRLF-based header termination

* **Supported Endpoints**

  * `/` → Basic 200 OK response
  * `/echo/<msg>` → Echo response
  * `/user-agent` → Returns client user-agent
  * `/files/<filename>` → File read/write support

* **GET & POST Support**

  * GET for fetching resources
  * POST for writing file contents

* **File Handling**

  * Static file serving
  * Binary-safe file reads/writes
  * Dynamic file path construction

* **Dynamic Buffer Management**

  * Custom resizable buffer (`dynarr`) for request/response handling

---

## 🧠 Key Concepts Implemented

* Low-level socket programming (`socket`, `bind`, `listen`, `accept`)
* Concurrent execution using `pthread`
* HTTP protocol parsing and response formatting
* Memory management using `malloc`, `realloc`, `free`
* File I/O using `fopen`, `fread`, `fwrite`
* Robust request handling using header-body separation

---

## 🏗️ Project Architecture

```
.
├── main.c           # Server setup, connection handling
├── parser.c         # HTTP request parsing logic
├── resp_build.c     # Response construction (echo, file, headers)
├── vector.c         # Dynamic buffer implementation
```

### Key Components

* **Connection Layer**

  * Accepts client connections and spawns threads

* **Request Handling**

  * Reads raw bytes from socket
  * Detects end of headers (`\r\n\r\n`)
  * Parses body using `Content-Length`

* **Routing Logic**

  * Matches request paths to handlers

* **Response Builder**

  * Generates HTTP-compliant responses with headers and body

---

## 🔄 Request Flow

1. Client connects via TCP
2. Server reads request into dynamic buffer
3. Headers parsed → body length determined
4. Request routed based on path
5. Response constructed
6. Response sent back to client

---

## 🧪 Example Requests

### Echo

```
GET /echo/hello HTTP/1.1
```

Response:

```
HTTP/1.1 200 OK
Content-Type: text/plain

hello
```

---

### File Read

```
GET /files/test.txt HTTP/1.1
```

---

### File Write

```
POST /files/test.txt HTTP/1.1
Content-Length: 5

hello
```

---

## 🛠️ Build & Run

```bash
gcc -o server main.c parser.c resp_build.c vector.c -lpthread
./server --directory ./data
```

Server runs on:

```
localhost:4221
```

---

## ⚡ Key Learnings

* Understanding of HTTP protocol internals
* Handling partial socket reads and request boundaries
* Managing concurrency with threads
* Designing modular and extensible system components
* Debugging low-level networking code

---

## 🚀 Future Improvements

* Thread pool instead of thread-per-client
* HTTP keep-alive support
* Improved error handling and logging
* Support for more HTTP methods and headers

---

## 📌 Summary

This project demonstrates building a fully functional HTTP server from scratch using low-level system primitives. It highlights strong understanding of networking, concurrency, and protocol design.
