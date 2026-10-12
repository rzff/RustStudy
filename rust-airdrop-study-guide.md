# Study Guide: Building an AirDrop-like Tool in Rust

**Student:** Ruben · **Format:** self-paced, mentor-reviewed · **Started:** October 2026

## Course overview

You will build `drop`, a tool that sends files between devices on the same local network: devices discover each other automatically, the receiver accepts or rejects, and the transfer is encrypted. It is AirDrop-*like*, not compatible with Apple's AirDrop (which relies on proprietary radio protocols).

**Starting point:** fluent Python, some Java and JavaScript, no Rust.

**End point:** you can design, write, test and refactor a multi-crate async Rust networking application, and explain every design decision in it.

### How each module works

1. **Read** the listed topics. Chapter numbers refer to your copy of *The Rust Programming Language*, 3rd edition ("the Book", also free at doc.rust-lang.org/book).
2. **Design note:** before coding, write 5 to 10 lines on how you plan to approach it. Send it for feedback.
3. **Exercises:** small, isolated drills. Do them in a scratch project.
4. **Build:** the module deliverable, added to the main project.
5. **Review:** submit the code. Feedback is sorted into *correct*, *incorrect*, and *unidiomatic*.
6. **Self-check:** answer the questions in your own words, without looking anything up.

### Ground rules

- You write all the code. Hints come in order: a question, then a docs pointer, then a sketch of the idea.
- Read compiler errors in full before asking for help.
- Every `.clone()` and `.unwrap()` gets a comment explaining why it is acceptable there.
- Run `cargo fmt` and `cargo clippy` before every submission.
- Commit to git after every exercise that works.

### Roadmap

| Module | Theme | Deliverable | Rough effort |
|--------|-------|-------------|--------------|
| 0 | Rust foundations | Local chunked file copier | 12 to 20 hours |
| 1 | TCP basics | Send one file between two terminals | 6 to 10 hours |
| 2 | Wire protocol | Handshake, metadata, accept/reject | 10 to 15 hours |
| 3 | Discovery | Devices find each other via mDNS | 8 to 12 hours |
| 4 | Async | Concurrent transfers, progress, cancel | 15 to 20 hours |
| 5 | Security | Encrypted channel, device identity | 12 to 18 hours |
| 6 | Robustness | Resume, folders, hostile input, tests | 10 to 15 hours |
| 7 | Front end (optional) | Terminal UI or desktop app | open-ended |

Effort figures are estimates for someone new to Rust; module 0 and module 4 vary the most.

---

## Module 0: Rust foundations

**Goal:** learn the ideas that have no equivalent in Python or Java, on a problem you will reuse in module 1.

### Learning objectives

- Use `cargo` to create, build, run, test, format and lint a project.
- Explain ownership, moves, and borrowing (`&` versus `&mut`).
- Handle absence and failure with `Option`, `Result` and `?`.
- Model data with structs and enums, and branch with `match`.
- Read and write files through the `Read` and `Write` traits.

### Reading

- Getting started and common concepts (Book ch. 1 to 3)
- Ownership, references, slices (ch. 4): read this one twice
- Structs (ch. 5), enums and `match` (ch. 6)
- Error handling (ch. 9)
- Modules (ch. 7) and testing (ch. 11) before exercise 0.6
- The I/O project (ch. 12) as a worked example

### Exercises

**0.1 Toolchain.** Install Rust with `rustup`, set up rust-analyzer in your editor, create a project, and run `cargo run`, `cargo check`, `cargo clippy`, `cargo fmt`.
*Done when:* you can explain what `check` does that `run` does not.

**0.2 Ownership drills.** Write a function that takes a `String` and returns its length. Call it, then try to print the original string afterwards. Make it compile three different ways: return the string back, borrow it, and clone it.
*Done when:* you can say which of the three you would choose and why.

**0.3 File inspector.** A command-line tool that takes a path argument and prints the file name and size in bytes. Handle: no argument given, file does not exist, path is a directory.
*Done when:* no input makes it panic.

**0.4 Human-readable sizes.** Add a function that turns a byte count into "1.4 MB". Write it as a pure function with no printing inside.
*Done when:* it is easy to test (you will test it in 0.6).

**0.5 Chunked copy.** Copy a file to a destination path by reading into a fixed-size buffer in a loop, and print percentage progress. Do not use `std::fs::copy` or read the whole file into memory.
*Done when:* a 1 GB file copies with flat memory use, and the copy is byte-identical.

**0.6 Structure and errors.** Split the code into functions and at least one module. Define your own error enum covering each failure case, and return it from your functions. Write tests for 0.4 and for one error path.
*Done when:* `main` contains almost no logic and `cargo test` passes.

### Deliverable

`drop copy <source> <destination>` with progress output, clean error messages and tests.

### Review checklist

- [ ] No panics on bad input
- [ ] Buffer is allocated once, not per loop iteration
- [ ] The final partial chunk is handled correctly
- [ ] Functions borrow where they do not need ownership
- [ ] Errors say what went wrong and on which path

### Self-check

1. Why does Rust have both `String` and `&str`?
2. What happens to a value when it is passed to a function by value?
3. Why can you have many `&` borrows or one `&mut` borrow, but not both?
4. What does `?` do, step by step?
5. `read` returned fewer bytes than the buffer size. Is the file finished?

---

## Module 1: TCP basics

**Goal:** move the copier across the network.

### Learning objectives

- Explain what a socket, a port, and a listener are.
- Understand that TCP delivers a stream of bytes, not messages.
- Write a blocking client and server with `std::net`.

### Reading

- `std::net` documentation: `TcpListener`, `TcpStream`
- Background: the TCP/IP model, the three-way handshake (any networking primer)

### Exercises

**1.1 Echo.** A server that accepts one connection and sends back whatever it receives. Test it with `nc` or `telnet` before writing a client.

**1.2 Client.** Write the matching client that sends a line and prints the reply.

**1.3 Observe the stream.** Have the client send three short writes in quick succession. Log how many reads the server needs and how large each is. Repeat with a large payload.
*Done when:* you can explain why the number of writes and reads do not match.

**1.4 File over TCP.** `drop receive <port>` and `drop send <ip:port> <file>`. The receiver writes everything it gets to a fixed output file until the sender closes the connection.

### Deliverable

A file transferred between two terminals, then between two machines on your network. Reuse the chunked loop from module 0.

### Review checklist

- [ ] Module 0 code was reused, not copied
- [ ] Read/write logic works against any `Read`/`Write`, not just files or sockets
- [ ] Connection errors are handled, not unwrapped

### Self-check

1. How does the receiver currently know the file is complete? What is fragile about that?
2. Why can the receiver not know the file name yet?
3. What would break if the sender wanted to send two files on one connection?

Your answers are the motivation for module 2.

---

## Module 2: A wire protocol

**Goal:** turn a raw byte stream into a conversation with defined messages.

### Learning objectives

- Design message framing with length prefixes.
- Model a protocol as enums and a state machine.
- Serialise structured data with `serde`.
- Reason about protocol versioning and malformed input.

### Reading

- Traits and generics (Book ch. 10, first two sections)
- `serde` documentation overview; compare a text format (JSON) with a binary one (`bincode` or `postcard`)
- Byte order (endianness) and why network protocols fix it

### Exercises

**2.1 Protocol on paper.** No code. Write the spec: list every message type, its fields, who sends it, and in which order. Draw the state machine for sender and receiver, including the reject and error paths. Submit this for review before 2.2.

**2.2 Framing.** Write `write_frame` and `read_frame`: a length prefix followed by that many bytes. Test them against an in-memory buffer, with no network involved.

**2.3 Messages.** Define a message enum (for example: hello, offer, accept, reject, chunk, done, error). Serialise it into frames and back. Write a round-trip test for every variant.

**2.4 Accept or reject.** The receiver shows the offered file name and size and asks the user. The sender waits for the answer.

**2.5 Integrity.** Send a hash of the file and verify it on the receiving side.

### Deliverable

`drop send` and `drop receive` speaking your protocol, with the spec committed as `PROTOCOL.md`.

### Review checklist

- [ ] The spec and the code agree
- [ ] A maximum frame size is enforced
- [ ] An unexpected message in a given state produces an error, not a panic
- [ ] The hello message carries a protocol version
- [ ] Protocol logic is testable without sockets

### Self-check

1. What happens if a peer sends a length prefix of 4 GB?
2. Why put a version number in the first message?
3. Why is an enum plus `match` a better fit here than a class hierarchy?
4. What did you choose for the chunk size, and what are the trade-offs?

---

## Module 3: Discovery

**Goal:** no more typing IP addresses.

### Learning objectives

- Explain UDP, broadcast and multicast, and how they differ from TCP.
- Explain what mDNS and DNS-SD do.
- Run background work on threads and communicate over channels.

### Reading

- Concurrency: threads, message passing, shared state (Book ch. 16)
- An overview of mDNS and DNS-SD (Bonjour/Zeroconf)
- Documentation for the `mdns-sd` crate

### Exercises

**3.1 Raw UDP.** Two programs: one broadcasts "I am here" every two seconds, the other listens and prints the sender's address. This is discovery by hand, so you understand what the library does for you.

**3.2 Advertise.** Use an mDNS crate to register a service for your tool, with a device name. Verify it with a system tool (`dns-sd -B` on macOS, `avahi-browse` on Linux).

**3.3 Browse.** `drop list` prints the devices currently visible, and removes ones that disappear.

**3.4 Integrate.** `drop send <file>` shows the discovered devices and lets you pick one. The receiver advertises itself while running.

### Deliverable

Sending a file by choosing a device name.

### Review checklist

- [ ] Discovery runs in the background without blocking the transfer
- [ ] Threads communicate through channels, or shared state is justified
- [ ] The device does not list itself
- [ ] The service is unregistered on shutdown

### Self-check

1. Why is discovery done over UDP and the transfer over TCP?
2. What do `Send` and `Sync` mean, and where did the compiler make you care?
3. Why does this not work across two different networks?

---

## Module 4: Async

**Goal:** handle many things at once: several transfers, discovery, user input, cancellation.

### Learning objectives

- Explain the difference between blocking I/O with threads and async I/O.
- Use `tokio`: tasks, async TCP, channels, `select!`.
- Share state safely with `Arc` and `Mutex`.
- Implement cancellation and backpressure.

### Reading

- Async programming (Book ch. 17)
- The official Tokio tutorial, start to finish
- Smart pointers: `Box`, `Rc`, `Arc` (Book ch. 15, selectively)

### Exercises

**4.1 Async echo.** Rewrite exercise 1.1 with `tokio`, handling many clients at once. Connect five clients simultaneously.

**4.2 Port the framing.** Make `read_frame` and `write_frame` async. Keep the tests passing.

**4.3 Port the transfer.** The receiver accepts multiple incoming transfers concurrently, one task each.

**4.4 Progress.** Transfer tasks report progress over a channel to a single task that prints it.

**4.5 Cancel.** Either side can cancel a transfer in progress; the other side is told, and partial files are cleaned up.

### Deliverable

A long-running `drop` process that advertises, receives several files at once, and can send while receiving.

### Review checklist

- [ ] No blocking calls inside async tasks
- [ ] No lock held across an `.await`
- [ ] Tasks end cleanly on cancel and on connection loss
- [ ] Channels are bounded, with a reasoned capacity

### Self-check

1. What is a future, and when does it actually run?
2. Why is holding a `Mutex` guard across an `.await` a problem?
3. What is backpressure, and where does your design apply it?
4. When would plain threads have been the better choice?

---

## Module 5: Security

**Goal:** nobody on the network can read or tamper with a transfer, and you know who you are talking to.

### Learning objectives

- Explain symmetric and asymmetric encryption, key exchange, and authenticated encryption.
- Compare TLS and the Noise protocol framework for peer-to-peer use.
- Implement device identity with trust on first use.
- Never write your own cryptography; use reviewed libraries.

### Reading

- A primer on Diffie-Hellman key exchange and authenticated encryption
- Noise protocol framework overview, or the `rustls` documentation
- How SSH handles "unknown host" prompts (trust on first use)

### Exercises

**5.1 Threat model.** No code. Write down who the attackers are (someone on the same Wi-Fi, a fake device with a copied name), what they can do today against your module 4 build, and what you will and will not defend against.

**5.2 See the problem.** Capture a transfer with Wireshark and find your file contents in the capture.

**5.3 Identity.** Generate a long-lived key pair on first run and store it. Show a short fingerprint in `drop list`.

**5.4 Encrypted channel.** Wrap the connection so all protocol messages are encrypted. Repeat 5.2 and confirm the contents are gone.

**5.5 Trust.** On first contact, show the peer's fingerprint and ask the user to confirm. Remember confirmed peers; warn loudly if a known name appears with a different key.

### Deliverable

Encrypted, authenticated transfers with a documented threat model in `SECURITY.md`.

### Review checklist

- [ ] No hand-rolled cryptographic primitives
- [ ] Private keys are stored with restrictive file permissions
- [ ] The protocol layer is unaware of whether the stream is encrypted
- [ ] Limitations are stated honestly in the threat model

### Self-check

1. What does encryption alone not protect against?
2. What is a man-in-the-middle attack, and which step of your design stops it?
3. Why is the file hash from module 2 not enough for integrity against an attacker?

---

## Module 6: Robustness

**Goal:** make it something you would trust with your own files.

### Learning objectives

- Design for partial failure.
- Treat all network input as hostile.
- Organise a workspace into library and binary crates.
- Write integration and property tests.

### Reading

- Cargo workspaces (Book ch. 14)
- Path traversal attacks
- `proptest` documentation

### Exercises

**6.1 Restructure.** Split into a `drop-core` library crate and a `drop-cli` binary crate. The library must not print or read from the terminal.

**6.2 Hostile names.** The sender offers a file named `../../.ssh/authorized_keys`. Make sure it cannot escape the download directory. Write the test first.

**6.3 Multiple files and folders.** Send several files and directory trees in one session.

**6.4 Resume.** If a transfer is interrupted, a retry continues from where it stopped.

**6.5 Timeouts.** A peer that connects and then goes silent does not hold resources forever.

**6.6 Property tests.** For any message, decoding its encoding gives the original. For any random bytes, decoding never panics.

**6.7 Integration test.** A test that starts a sender and receiver in one process and transfers a real file end to end.

### Deliverable

Version 1.0 with a README, tests in CI, and a public repository.

### Self-check

1. Which of your invariants are enforced by types, and which only by tests?
2. What is the worst thing a malicious peer can still do to your receiver?

---

## Module 7 (optional): Front end

Pick one, built only on the `drop-core` public API:

- **Terminal UI** with `ratatui`: device list, progress bars, accept prompts.
- **Desktop app** with Tauri: drag and drop, notifications.

If the front end needs something the core does not expose, change the core's API deliberately; do not reach around it.

---

## Stretch topics

- Transfer over QUIC instead of TCP
- Compression, and when it does not help
- Bluetooth LE for discovery, as real AirDrop does
- Reading the OpenDrop project to see how Apple's protocol actually works
- Benchmarking and profiling throughput

## Progress log

| Module | Started | Submitted | Reviewed | Notes |
|--------|---------|-----------|----------|-------|
| 0 | | | | |
| 1 | | | | |
| 2 | | | | |
| 3 | | | | |
| 4 | | | | |
| 5 | | | | |
| 6 | | | | |
| 7 | | | | |
