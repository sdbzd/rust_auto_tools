# json_typegen_shared

[json_typegen](https://typegen.vestera.as/) as just a library,
for use in build scripts and other crates.
If you want an actual interface, like a website, CLI or procedural macro, check the repo:
[github.com/evestera/json_typegen](https://github.com/evestera/json_typegen)

Note: This crate is to a certain extent considered internal API of the `json_typegen` tools.
If you want to use this crate directly, be prepared for breaking changes to happen, and consider
[opening an issue](https://github.com/evestera/json_typegen/issues/new)
to let me know what you are using. (Breaking changes may still happen,
but then I'll at least try to keep your use-case in mind if possible.
This has happened enough by now that there are parts I already consider public API.)

## Crate feature flags

All of these flags are on by default,
but you can avoid dependencies you don't care about disabling some or all of them,
with e.g. something like this to only enable option parsing:

```
json_typegen_shared = { version = "*", default-features = false, features = ["option-parsing"] }
```

### `remote-samples`

Required to load samples from URLs.

### `local-samples`

Required to load samples from local paths.

### `option-parsing`

Required to parse an options object from a string.
Since this is required for code generation from macro-like strings,
this is also required for the functions `codegen_from_macro` and `codegen_from_macro_input`.
