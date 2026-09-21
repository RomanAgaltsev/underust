# Changelog

## [0.2.0](https://github.com/RomanAgaltsev/underust/compare/v0.1.0...v0.2.0) (2026-09-21)


### Features

* **cli:** ci-stubs, prove and radar-check gates, and the 1.98 radar entry ([d9fbf5d](https://github.com/RomanAgaltsev/underust/commit/d9fbf5df298803dfcc26d758413df90e7a6b6c01))
* **cli:** doctor reports gradeable tracks and names every remedy ([4168ac8](https://github.com/RomanAgaltsev/underust/commit/4168ac8e000024e02bf19798c94022468227d1d6))
* **cli:** hint, reveal with a gate that refuses, and progress ([76e257b](https://github.com/RomanAgaltsev/underust/commit/76e257b2c6cbcbff0f06d5cde9bb79a7712df2da))
* **cli:** predict scaffolding and prediction-vs-reality grading ([f8735a4](https://github.com/RomanAgaltsev/underust/commit/f8735a4e08187972af3ce639c005191edabee99d))
* **cli:** test command with requires refusal and constrain ban enforcement ([b55dd36](https://github.com/RomanAgaltsev/underust/commit/b55dd36fec59995bc97615da67cc9012f5ca0602))
* **cli:** underust binary with list and validate ([07db40e](https://github.com/RomanAgaltsev/underust/commit/07db40ecccccc8dcf669916659268e263daf4cd2))
* **core:** ast-based forbidden-construct checker for constrain tasks ([9caebad](https://github.com/RomanAgaltsev/underust/commit/9caebad9694d027fd86a36a419df5dabe9397d5f))
* **core:** deterministic gzip+base64 seal with pinned golden bytes ([069d8fe](https://github.com/RomanAgaltsev/underust/commit/069d8fec272ce21c4b418a7e32d50244e99eed21))
* **core:** host probe parsers for rustc, rustup and toolchains ([a9b9e6b](https://github.com/RomanAgaltsev/underust/commit/a9b9e6b1caf81eed9fc073d9d233f34373037072))
* **core:** local progress store for the hint and reveal ladder ([0ea9e7f](https://github.com/RomanAgaltsev/underust/commit/0ea9e7fcc8b4dd40094796e1f79e3b203f63a2f0))
* **core:** one-line measurement protocol and prediction diffing ([8f29e5b](https://github.com/RomanAgaltsev/underust/commit/8f29e5b48b5c33d629e939f184627434bb813fe8))
* **core:** resolve task requirements against the host, with remedies ([61b4f4b](https://github.com/RomanAgaltsev/underust/commit/61b4f4b6b0b7408c4d133ecf39e65a197a232757))
* **core:** task.toml schema, loader and validation ([dafda91](https://github.com/RomanAgaltsev/underust/commit/dafda910ee15ad3158e12549c87d2fd5edf16c36))
* **grade:** counting global allocator and refcount probe ([146d72c](https://github.com/RomanAgaltsev/underust/commit/146d72c164a235d566ffc7bcd520f8262574699e))
* **grade:** drop log grader and the emit measurement channel ([4e75f7e](https://github.com/RomanAgaltsev/underust/commit/4e75f7ef21462a1b682542300268e17fb7ea64a0))
* **tasks:** alloc/01-allocation-count, the optimize exemplar graded on allocations ([a7bbc3d](https://github.com/RomanAgaltsev/underust/commit/a7bbc3dd5bccd684659f42eaf26155eb848ba2cc))
* **tasks:** drop/01-field-order, the predict exemplar ([b464e4b](https://github.com/RomanAgaltsev/underust/commit/b464e4b69d230596220214c78134e0816c60089d))
* **tasks:** own/01-no-clone, the constrain exemplar with an enforced ban ([0a998df](https://github.com/RomanAgaltsev/underust/commit/0a998df4dbea2b12348f64b687c8349826e5d222))
* **tasks:** unsafe/01-aliasing, the soundness exemplar, with pinned test digest ([32a482a](https://github.com/RomanAgaltsev/underust/commit/32a482a9f66d7ca0a056c689f7ec24b29d2780da))
* **tasks:** weak/01-break-the-cycle, the build exemplar, with the first seal ([be44c83](https://github.com/RomanAgaltsev/underust/commit/be44c831646826a9c4341947b6d66c6233b7e5b1))


### Bug Fixes

* **cli:** drop repo helpers nothing uses yet ([a872c52](https://github.com/RomanAgaltsev/underust/commit/a872c520752403bbd8c3c408838dd20f3faae987))
* **cli:** lay work/ over the task directory so solving actually works ([bfb11ce](https://github.com/RomanAgaltsev/underust/commit/bfb11cedc240283c4501d62e794a9ce6c1762dde))
* **core:** drop the unused Spanned import ([ad96231](https://github.com/RomanAgaltsev/underust/commit/ad96231dbef76705ce9584b2ee8c46e0b0159f00))
* **release:** release-type simple, because the root is a virtual workspace ([f450806](https://github.com/RomanAgaltsev/underust/commit/f4508061df62cdd2da4e8faed5b3b849ef62ff6e))
