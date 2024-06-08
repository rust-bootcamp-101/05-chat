### Cargo deny 查询错误日志
由于默认输入的日志太多了，需要过滤(否则terminal看不到错误的位置)
```bash
cargo deny --log-level error
```

### Cargo deny 检测到错误，如何处理
如下错误，提示某个库可能会遭受到攻击，不能使用该库(jwt-simple引用到了rsa这个库)
则需要考虑不要使用这个库，或如果你仍要使用这个库，确认没有使用到这个rsa，则可以把 ID: RUSTSEC-2023-0071 加入到deny的过滤中
```
error[vulnerability]: Marvin Attack: potential key recovery through timing sidechannels
    ┌─ /Users/yourname/code/rust/chat/Cargo.lock:165:1
    │
165 │ rsa 0.9.6 registry+https://github.com/rust-lang/crates.io-index
    │ --------------------------------------------------------------- security vulnerability detected
    │
    = ID: RUSTSEC-2023-0071
    = Advisory: https://rustsec.org/advisories/RUSTSEC-2023-0071
    = ### Impact
      Due to a non-constant-time implementation, information about the private key is leaked through timing information which is observable over the network. An attacker may be able to use that information to recover the key.

      ### Patches
      No patch is yet available, however work is underway to migrate to a fully constant-time implementation.

      ### Workarounds
      The only currently available workaround is to avoid using the `rsa` crate in settings where attackers are able to observe timing information, e.g. local use on a non-compromised computer is fine.

      ### References
      This vulnerability was discovered as part of the "[Marvin Attack]", which revealed several implementations of RSA including OpenSSL had not properly mitigated timing sidechannel attacks.

      [Marvin Attack]: https://people.redhat.com/~hkario/marvin/
    = Announcement: https://github.com/RustCrypto/RSA/issues/19#issuecomment-1822995643
    = Solution: No safe upgrade is available!
    = rsa v0.9.6
      └── superboring v0.1.2
          └── jwt-simple v0.12.9
              └── chat-server v0.1.0
```
把 ID: RUSTSEC-2023-0071 加入到deny的过滤中
```bash
# A list of advisory IDs to ignore. Note that ignored advisories will still
# output a note when they are encountered.
ignore = [
  #"RUSTSEC-0000-0000",
  "RUSTSEC-2023-0071"
]
```
