[![progress-banner](https://backend.codecrafters.io/progress/http-server/cb7cf933-1eb8-417d-a114-99161c202c0b)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own HTTP server" Challenge](https://app.codecrafters.io/courses/http-server/overview).

# High-Performance HTTP/1.1 Web Server in Rust

A custom, multi-threaded HTTP/1.1 web server built from the ground up in Rust, designed to explore high-performance networking, concurrency, and I/O handling. This project showcases building a robust web server from raw TCP sockets, featuring a custom HTTP parser, static file serving, and dynamic GZIP compression.

To quantify its capabilities and understand architectural trade-offs, the server underwent a series of rigorous benchmarks and stress tests, comparing its performance directly against a Node.js server.

## 🚀 Performance Benchmarks & Key Learnings

The tests consistently demonstrated the Rust server's superior performance and resilience, offering valuable insights into system-level programming and concurrency models.

### 1. Standard Throughput Test (10KB file, 100 concurrent clients)

This initial benchmark measured the raw requests per second (RPS) both servers could handle when serving a small static file.

<img width="256" height="256" alt="Gemini_Generated_Image_m2cxipm2cxipm2cx" src="https://github.com/user-attachments/assets/e92a6be4-5103-4872-8ed1-f24b6ff6d5ae" />


| Server | Requests/sec |
| :--- | :--- |
| Rust Server | ~6,709 |
| Node.js | ~2,353 |

**Learning:** The Rust server achieved **2.8x higher throughput** than Node.js. This highlights Rust's advantage as a compiled language, executing native code directly on the CPU with minimal runtime overhead, compared to Node.js's JavaScript engine.

### 2. Heavy Load Stress Test (10KB file, 500 concurrent clients)

This test pushed both servers with a higher number of concurrent connections to assess their stability and latency under stress.

<img width="256" height="256" alt="Gemini_Generated_Image_5m54we5m54we5m54" src="https://github.com/user-attachments/assets/46446494-6ac9-450a-afc0-2ab361315226" />


| Metric | Rust Server | Node.js Server | Result |
| :--- | :--- | :--- | :--- |
| Throughput (RPS) | ~4,840 | ~3,154 | **53% Higher RPS** |
| Average Latency | 20.6 ms | 41.0 ms | **2x Lower Latency** |
| 99th Percentile Latency | 27.6 ms | 54.3 ms | **2x More Consistent** |

**Learning:** Even under heavy load, the Rust server sustained significantly higher throughput and delivered a consistently faster user experience. This demonstrates the effectiveness of Rust's true multi-threading model (thread-per-client) in achieving parallel execution, contrasting with Node.js's single-threaded event loop which, while efficient, eventually hits CPU-bound limits under intense computational or management overhead.

### 3. Large File I/O Throughput Test (50MB file, 500 concurrent clients)

This test focused on data transfer capabilities, pushing the limits of disk I/O and network bandwidth.

<img width="256" height="256" alt="Gemini_Generated_Image_m2cxipm2cxipm2cx (2)" src="https://github.com/user-attachments/assets/14066c93-7e70-4508-a344-a68add5f61e7" />


| Metric | Rust Server |
| :--- | :--- |
| Data Throughput | **1.27 GiB/s** |

**Learning:** The Rust server achieved a massive throughput of **1.27 Gigabytes per second**. This indicates an extremely efficient I/O pipeline, where the software overhead is minimal, and performance is primarily bottlenecked by the underlying hardware (SSD read speed and network interface), rather than the server's code.

### 4. Hardcore Stability Test (50MB file, 4,000 concurrent clients)

<img width="256" height="256" alt="Gemini_Generated_Image_m2cxipm2cxipm2cx (1)" src="https://github.com/user-attachments/assets/cec0b2f5-b38a-402f-acbf-94f3f2fc813d" />


| Metric | Rust Server | 
| :--- | :--- |
| Success Rate | **100%** |
| Sustained Throughput | **1.17 GiB/s** |

**Learning:** Despite the immense pressure of 4,000 simultaneous clients each downloading a 50MB file, the Rust server maintained a **100% success rate** and sustained an incredible data transfer rate. This highlights the inherent stability and resource efficiency of Rust applications, even with a thread-per-client model that can become latency-bound under such conditions. It demonstrated that the server would not crash or drop connections, proving its fundamental robustness.



## 💡 Architectural Insights & Future Directions

This project successfully demonstrates that a custom-built Rust server can deliver exceptional performance and stability, often surpassing higher-level runtimes in raw speed and efficiency. The "thread-per-client" model proved surprisingly robust for I/O-bound tasks and heavy loads.

However, the benchmarks also illuminated the architectural trade-offs: while highly stable, the latency for individual requests increased under extreme concurrency as thousands of threads contended for CPU resources. This finding provides a data-driven rationale for the next steps: exploring more advanced concurrency patterns.

Future improvements include:
* Implementing a **thread pool** to manage a fixed set of worker threads more efficiently.
