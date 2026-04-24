# Spec Title — Short Descriptor

## Goal

[One or two sentences: what does this spec achieve and why.]

---

## Small Change (bug fix, tweak)
Skip this template. Write a clear PR description instead.

---

## 1. Section Name

**What:** [One sentence describing the deliverable.]

**Spec:**
- Requirement 1
- Requirement 2
- Requirement 3

**Checkpoint:** [Concrete, verifiable condition that confirms this section is done.]

---

## 2. Section Name

**What:** [One sentence describing the deliverable.]

**Spec:**
- Requirement 1
- Requirement 2

**Checkpoint:** [Concrete, verifiable condition.]

---

## Implementation Order

1. Section that must come first (e.g., audit / discovery)
2. Section that depends on 1
3. Section that depends on 2
4. Final pass (fmt, clippy, test, version bump if applicable)

## Test Plan

- [Test category]: [what is tested]
- [Test category]: [what is tested]
- `cargo fmt && cargo clippy -- -D warnings && cargo test` must pass

## Files to Modify

- [ ] file.rs
- [ ] CHANGELOG.md
- [ ] README.md (if user-facing)
