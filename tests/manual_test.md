# Manual Test Steps for Session Start

1. Build the project:
```bash
cargo build
```

2. Start a new session with default settings:
```bash
cargo run -- session start
```
Expected: 
- Session starts with a random name
- Displays prompt for input
- Can enter messages and get responses
- Ctrl+C interrupts gracefully
- Empty input exits

3. Start a session with specific name:
```bash
cargo run -- session start test_session
```
Expected:
- Session starts with name "test_session"
- All other behavior same as default

4. Start a session with profile:
```bash
cargo run -- session start --profile default
```
Expected:
- Session starts with specified profile
- Profile settings are applied

5. Test interruption handling:
- Start a session
- Type a message but interrupt with Ctrl+C before response
- Verify graceful handling and recovery message

6. Test statistics:
- Complete a session with several messages
- Check logs in ~/.config/goose/logs
- Verify token usage and cost tracking