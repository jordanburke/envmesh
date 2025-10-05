# Claude Code Handoff - EnvMesh

**For Claude Code Sessions:** Read this first when opening this project fresh!

## TL;DR

This is **EnvMesh** - a P2P environment variable sync tool built with Rust + Tauri + libp2p.

**Current State:** 🟡 Project structure complete, core modules stubbed, needs implementation

**Your First Actions:**
1. Read this file (you're doing it!)
2. Read `CLAUDE.md` - Architecture & modules
3. Read `STATUS.md` - What's done, what's next
4. Run `cd src-tauri && cargo check` - See current compilation state

## Context Chain

```
~/.claude/CLAUDE.md          → Global standards (auto-loaded)
    ↓
CLAUDE.md                    → Project architecture
    ↓
STATUS.md                    → Implementation status
    ↓
HANDOFF.md (this file)       → Session continuity
```

## What We Built So Far

### Session 1 (2025-10-05)
**Goal:** Create native P2P env var sync app

**Decisions Made:**
- ✅ Tauri 2.0 over Electron (size: 3-10MB vs 85MB)
- ✅ libp2p for P2P mesh networking
- ✅ Gossipsub for message distribution
- ✅ SQLite for local storage (encrypted at app layer)
- ✅ CRDT (automerge) for conflict resolution
- ✅ Name: EnvMesh (EnvSync was taken)

**What Was Built:**
```
envmesh/
├── CLAUDE.md       ← Architecture, modules, dev guide
├── STATUS.md       ← Implementation checklist
├── README.md       ← User documentation
├── src-tauri/      ← Rust backend (6 modules)
│   ├── main.rs     ← Tauri app + system tray
│   ├── p2p.rs      ← libp2p networking
│   ├── storage.rs  ← SQLite storage
│   ├── crypto.rs   ← AES-256-GCM encryption
│   ├── api.rs      ← Tauri commands
│   └── cli.rs      ← CLI interface
└── ui/             ← Frontend (HTML/CSS/JS)
```

**Current State:**
- All modules exist with function signatures
- Implementations are stubs (return errors)
- Does NOT compile yet (known issues in STATUS.md)
- Git initialized with first commit

## Critical Context

### Why This Project?
User wanted OS env var sync across machines (Windows/Linux/macOS) with these requirements:
- ❌ NOT file-based (like dotfiles)
- ❌ NOT paid service
- ✅ P2P or server-client architecture
- ✅ Native application (not Electron)

We chose P2P mesh over server-client because:
- No single point of failure
- Works offline (local-first)
- No server to maintain
- Truly distributed

### Key Architectural Choices

**Tauri + Web UI** (not pure Rust GUI)
- Easier UI development (HTML/CSS/JS)
- Still native performance
- 3-10MB installer size

**libp2p Gossipsub** (not custom protocol)
- Battle-tested (IPFS, Ethereum use it)
- Built-in peer discovery, NAT traversal, encryption
- Automatic mesh healing

**CRDT** (not last-write-wins)
- Conflict-free merging
- No coordination needed
- Works offline

**Encrypted at App Layer** (not SQLCipher)
- More control over encryption
- Cross-platform compatibility
- Can encrypt individual values differently

## What Needs Attention

### Immediate Next Steps (Phase 1)
1. **Fix compilation**
   - Tauri 2.x API compatibility
   - Add shared app state
   - Minimal implementations to compile

2. **Storage layer**
   - Complete CRUD operations
   - Add encryption calls
   - Basic tests

3. **CLI tool**
   - Implement get/set/list
   - Shell export formatting
   - Test local-only workflow

### Medium-Term (Phase 2)
- libp2p peer discovery working
- Basic broadcast/receive
- 2-machine sync demo

### Long-Term (Phase 3)
- CRDT conflict resolution
- Cross-platform packages
- Production polish

## Known Traps & Gotchas

### Tauri 2.x Changes
- `system-tray` feature → `tray-icon` feature ✅ (already fixed)
- System tray API changed significantly
- Window management API different from v1.x

### libp2p Version
- Using v0.54
- APIs change frequently
- May need version pinning

### CRDT Integration
- Automerge v0.5 specified
- Needs careful integration with storage layer
- Tombstones need cleanup strategy

## If You Get Stuck

### Compilation Errors
1. Check `STATUS.md` - Known Issues section
2. Run `cargo check` for specific errors
3. Tauri docs: https://v2.tauri.app/

### Architecture Questions
1. Read `CLAUDE.md` - Module descriptions
2. Check this file - "Critical Context"
3. Look at module comments in code

### "What do I work on?"
1. Check `STATUS.md` - Next Steps section
2. Follow Phase 1 → Phase 2 → Phase 3
3. Each phase builds on previous

## Development Workflow

```bash
# Standard workflow
cd src-tauri
cargo check       # Quick compile check
cargo build       # Full build
cargo run         # Run the app

# With logging
RUST_LOG=debug cargo run

# Tests (when they exist)
cargo test

# Format + lint
cargo fmt
cargo clippy
```

## Important Files to Read

**Must Read:**
1. `CLAUDE.md` - Architecture (10 min read)
2. `STATUS.md` - Current state (5 min read)
3. This file (you're reading it!)

**Reference:**
- `README.md` - User-facing docs
- `~/.claude/CLAUDE.md` - Global standards (auto-loaded)

**Code:**
- `src-tauri/src/main.rs` - Start here
- `src-tauri/src/p2p.rs` - P2P logic
- `src-tauri/src/storage.rs` - Database

## Session Continuity Tips

### Starting a New Session
1. Run `git log --oneline` - See what was done
2. Run `git status` - Check for uncommitted work
3. Read `STATUS.md` - Where we left off
4. Check `cargo check` - Current build state

### Before Ending a Session
1. Update `STATUS.md` if you completed major items
2. Commit your work with descriptive messages
3. Note any blockers or decisions needed

### Mid-Implementation?
- Leave detailed comments about what you were doing
- Update STATUS.md with current progress
- Consider creating a checkpoint commit

## Questions You Might Have

**Q: Can I change the architecture?**
A: Check with user first. Current choices were deliberate based on requirements.

**Q: Should I add more dependencies?**
A: Prefer using existing ones. If needed, justify in commit message.

**Q: Where do I put X?**
A: Follow the module structure in CLAUDE.md. When in doubt, ask.

**Q: Why is X stubbed?**
A: We built structure first, implementation second. It's intentional.

**Q: Do I need to read everything?**
A: Minimum: this file + CLAUDE.md. STATUS.md is very helpful too.

## Success Criteria

**Phase 1 Success:**
- Project compiles
- Can set/get env vars locally
- Can export to shell
- Persists across restarts

**Phase 2 Success:**
- 2 machines discover each other
- Changes sync between them
- Works on local network

**Phase 3 Success:**
- Cross-platform packages
- Auto conflict resolution
- Production ready

## Final Notes

This project is well-architected and documented. The hard design decisions are done. Now it's about implementing the vision.

**Don't panic if it doesn't compile yet** - that's expected and documented.

**Follow the phases** - Each builds on the previous.

**Ask questions** - The user (Jordan) is available for clarifications.

Good luck! 🚀

---

*Last updated: 2025-10-05*
*Next session: Start with Phase 1 - Fix compilation*
