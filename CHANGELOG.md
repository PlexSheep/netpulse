# Changelog

## [Unreleased]

## [0.5.0](https://github.com/PlexSheep/netpulse/compare/v0.4.1...v0.5.0)

### 📚 Documentation

- *(api)* Fix examples for no default options - ([1db6d5b](https://github.com/PlexSheep/netpulse/commit/1db6d5be018acca636e0f8fb4997bc0b6dab0850))

### ⚙️ Miscellaneous Tasks

- Docs.rs should show all features - ([14c781f](https://github.com/PlexSheep/netpulse/commit/14c781f6d32f493808ff1ef26932e592fa5fad30))
- Test without default features - ([cd33873](https://github.com/PlexSheep/netpulse/commit/cd33873c5215b2c882ff3f5860acef2d3fa49cc1))


## [0.4.1](https://github.com/PlexSheep/netpulse/compare/v0.4.0...v0.4.1)

### ⛰️ Features

- Consider an environment variable when initializing the logging - ([0142cc5](https://github.com/PlexSheep/netpulse/commit/0142cc553b93392e84cd66e2475836647d8007c7))

### ⚙️ Miscellaneous Tasks

- Run doctests too in ci - ([ef3703a](https://github.com/PlexSheep/netpulse/commit/ef3703a44ed21c0164d2bb6efcf2d7a1600f4789))


## [0.4.0](https://github.com/PlexSheep/netpulse/compare/v0.3.0...v0.4.0)

### ⛰️ Features

- *(analyze)* Dump the entire store (it's checks) [#8](https://github.com/PlexSheep/netpulse/pull/8) - ([5770e01](https://github.com/PlexSheep/netpulse/commit/5770e019942ad4905233ae7ec6dc38de0f348b61))
- *(analyze)* Show size of store in mem and fs + ratio - ([d12638b](https://github.com/PlexSheep/netpulse/commit/d12638b71194bc3738b0047e6bfb1753aeeffd86))
- *(analyze)* Print the store version of in memory store - ([2dd766d](https://github.com/PlexSheep/netpulse/commit/2dd766d1e793edd55a998fd6424ca11eb796fd80))
- *(daemon)* Have setup ask to execute the systemd stuff for the user - ([643a282](https://github.com/PlexSheep/netpulse/commit/643a2826fd72cea2271bfc781f242f07b8a103b2))
- *(daemon)* Reload the store on SIGHUP - ([235d250](https://github.com/PlexSheep/netpulse/commit/235d2508620fca038f2fb878235ab4441a324121))
- *(reader)* Dump all and dump only failed - ([c52f4e3](https://github.com/PlexSheep/netpulse/commit/c52f4e344dbb5d5f9b387a92067a0319cbb85672))
- *(setup)* Stop the netpulsed service at the start of setup - ([9a5fdd0](https://github.com/PlexSheep/netpulse/commit/9a5fdd04521bfe1d897d633cf6e11633572965eb))
- Use logging with tracing for everything in the library and set it up for the executables [#5](https://github.com/PlexSheep/netpulse/pull/5) - ([450fd05](https://github.com/PlexSheep/netpulse/commit/450fd05b6179c0fb8630b64d593de65fe589fae4))

### 🐛 Bug Fixes

- *(daemon)* Stop_requested was initialized with true - ([d3d00f6](https://github.com/PlexSheep/netpulse/commit/d3d00f639a8f5c764c05888ef21522e944978782))
- *(setup)* Args need to be split - ([ca56268](https://github.com/PlexSheep/netpulse/commit/ca56268264937f1ec4aee941658f474ed03818ad))
- *(setup)* Stop the running service first - ([8b56aa1](https://github.com/PlexSheep/netpulse/commit/8b56aa12f5d15940cb12603ba972cfd3b1f220d9))
- Logging in common/netpulsed - ([9f54039](https://github.com/PlexSheep/netpulse/commit/9f540394c248a2798255ed5132fde7423df98d05))

### 🚜 Refactor

- *(analyze)* Improve display functions - ([561f2ff](https://github.com/PlexSheep/netpulse/commit/561f2ffa7e8bcdfa771c8dca052e20bcc48895ce))
- *(bins)* Share some code in the new common module - ([6cfca2b](https://github.com/PlexSheep/netpulse/commit/6cfca2b14e82d1e433581fc96d4219d5a97c96c5))
- *(daemon)* Better error handling in main - ([8cd29a2](https://github.com/PlexSheep/netpulse/commit/8cd29a27d8159cadc7614067aa1aca7201d19680))
- *(records)* Display_groups moved to records, better display - ([7235105](https://github.com/PlexSheep/netpulse/commit/723510526a810785b9e95eed0e49a83ed47864fa))
- *(setup)* More debug prints for the systemd setup - ([0714ebf](https://github.com/PlexSheep/netpulse/commit/0714ebf5c6794e33cabe3e8edfeb3ebb0ea0213a))

### 📚 Documentation

- *(api)* Fix test - ([0005078](https://github.com/PlexSheep/netpulse/commit/000507824c6c8825d9279990be38fdc39abe0fd6))

### ⚙️ Miscellaneous Tasks

- Automatic Rust CI changes - ([56dd8ce](https://github.com/PlexSheep/netpulse/commit/56dd8ceb6413c1e588b879043cc771404e57296a))
- Fix pedantic warnings - ([cc52d70](https://github.com/PlexSheep/netpulse/commit/cc52d7068ce6317dbde1c1e63406423b5c1936e4))


## [0.3.0](https://github.com/PlexSheep/netpulse/compare/v0.2.0...v0.3.0)

### ⛰️ Features

- *(daemon)* Add setup flag and make the daemon flag official - ([8a324c2](https://github.com/PlexSheep/netpulse/commit/8a324c2e2181d6b65f3b704236d20f37e9078f8a))
- *(store)* Return the new checks from make_checks and let the daemon print them - ([d82e852](https://github.com/PlexSheep/netpulse/commit/d82e85206e3fff217704dbffb93d0722bc2af28c))
- *(systemd)* Copy the netpulsed to /usr/local/bin/ in the setup - ([98c2aea](https://github.com/PlexSheep/netpulse/commit/98c2aead725647443f6da15d9c07a90f61cdb8f7))
- *(systemd)* Only remove pidfile if it's a manual daemon - ([61b243b](https://github.com/PlexSheep/netpulse/commit/61b243bc325028b0fddcca540edb384b45a88ba5))
- *(systemd)* Install service file - ([7b3c9e9](https://github.com/PlexSheep/netpulse/commit/7b3c9e948d575e07c984ea27295ac4148e191f38))
- *(systemd)* Add netpulsed.service file - ([ffe7985](https://github.com/PlexSheep/netpulse/commit/ffe7985f198dd99ad512f1c0b542cc6678160f7f))
- Default enable icmp again, as CAP_NET_RAW is okay with systemd - ([4617c85](https://github.com/PlexSheep/netpulse/commit/4617c85d9be94ffff1af8e7844045a753042530a))

### 🐛 Bug Fixes

- *(analyze)* Check for ip flag needs to be inverted - ([a08854d](https://github.com/PlexSheep/netpulse/commit/a08854df3b349af8a438877b99afa379bcff9b07))
- *(daemon)* Setup copy was missing the bin name - ([87b6b2e](https://github.com/PlexSheep/netpulse/commit/87b6b2e57afffcad7c123ca933e80c51f870af69))

### 🚜 Refactor

- *(analyze)* Make analyze functions more generic and add more parts - ([0b3155b](https://github.com/PlexSheep/netpulse/commit/0b3155bb67f14d519c0d27dccea2994836c1709e))
- *(daemon)* Consolidate setup and setup-systemd into just setup - ([ab3ebfb](https://github.com/PlexSheep/netpulse/commit/ab3ebfbf40650dab373f935545da850253d430ef))
- Refactor logs, primitive_make_checks - ([1698378](https://github.com/PlexSheep/netpulse/commit/1698378789706d42ec06b37afb6a8f6c223d75bc))
- Rename DaemonError to RunError and print netpulse -t data with group_display - ([edce0bb](https://github.com/PlexSheep/netpulse/commit/edce0bb4865fa56f5e6e59c96d5924b2dba48473))

### 📚 Documentation

- Add setup and update sections for the daemon to the readme - ([0dc80f0](https://github.com/PlexSheep/netpulse/commit/0dc80f0443922fe2612cdf1ed406b6cb796dc6bf))

### ⚙️ Miscellaneous Tasks

- Automatic Rust CI changes - ([b44b084](https://github.com/PlexSheep/netpulse/commit/b44b0840b8778d0fdc8ca3f88fdc7e57af7bedb6))
- Remove unused import - ([624e8bd](https://github.com/PlexSheep/netpulse/commit/624e8bd0e49697f5c1a5bd32391c9663d7010edb))


## [0.2.0](https://github.com/PlexSheep/netpulse/compare/v0.1.0...v0.2.0)

### ⛰️ Features

- *(analyze)* Add ipv4 and ipv6 section - ([78c6575](https://github.com/PlexSheep/netpulse/commit/78c657535ee48f2b2144174d55b1012a8f1b7fcc))
- *(checks)* Ip_type and get_hash - ([90c691a](https://github.com/PlexSheep/netpulse/commit/90c691ab8e378baaa46a57280c4b8cd771fe5e2b))
- *(error)* Add AmbiguousFlags and MissingFlags errors - ([a5dedf9](https://github.com/PlexSheep/netpulse/commit/a5dedf90d6c91d23bef7cf582e3174b9cd2259f0))

### 🐛 Bug Fixes

- *(cli)* Print usage when bad options are given - ([3f2cd27](https://github.com/PlexSheep/netpulse/commit/3f2cd270f678de05640dcbfaa49c8851bee0c572))
- *(cli)* Don't panic when a bad option is passed - ([599ba77](https://github.com/PlexSheep/netpulse/commit/599ba77116ec5dd3dd0a9ff54011cd070852dfb0))
- *(daemon)* Daemon paniced when loading failed - ([4337bc3](https://github.com/PlexSheep/netpulse/commit/4337bc3cb620902a75c14e0d152410450d081667))
- *(store)* Chown instead of chmod - ([99135a0](https://github.com/PlexSheep/netpulse/commit/99135a08fbe91d08b269ccab62b0ee7305e7e310))
- *(store)* Setperms fails, print more info - ([dd8465f](https://github.com/PlexSheep/netpulse/commit/dd8465fa83fa35112179d8db65a645a54c7f7b8b))
- *(store)* Chown store directory after creating it - ([dbd7eb3](https://github.com/PlexSheep/netpulse/commit/dbd7eb36645ef5a8ff562d54430e485464bbdaca))
- Show source for errors - ([a8adacc](https://github.com/PlexSheep/netpulse/commit/a8adacc6ea8fc21e3cff6139aea517b72ec5d3d8))

### 🚜 Refactor

- *(cli)* Remove daemon --fail - ([d99d5f6](https://github.com/PlexSheep/netpulse/commit/d99d5f6aa42866022fd430db8eadafa5970294e5))
- *(daemon)* Cleanup when store load does not work - ([50097df](https://github.com/PlexSheep/netpulse/commit/50097df0a0aa4dba6ec2af1373ed1b43dd48153f))
- *(store)* Add a setup function so that the setup can run as root - ([51edeab](https://github.com/PlexSheep/netpulse/commit/51edeab48b539f93eae93757d0638e83de720a84))
- *(store)* Print additional err messages in store create - ([9b92c88](https://github.com/PlexSheep/netpulse/commit/9b92c88955be83f64aeee57d0ceba24b679c2471))

### 📚 Documentation

- *(api)* Fix Store::setup example - ([9d7792f](https://github.com/PlexSheep/netpulse/commit/9d7792f177105355406a9417242a9447d75e34d5))

### ⚙️ Miscellaneous Tasks

- Automatic Rust CI changes - ([526f2d2](https://github.com/PlexSheep/netpulse/commit/526f2d2eac3d018c6bdc4a605f263ae075995288))


## [0.1.0]

### ⛰️ Features

- *(analyze)* Add hash to store metadata - ([a9de51a](https://github.com/PlexSheep/netpulse/commit/a9de51a784e470deb057a9e2bec1383a26effabe))
- *(analyze)* Check if any outages exist before analyzing for them - ([3ba8085](https://github.com/PlexSheep/netpulse/commit/3ba8085312105022cf9f61d254c067cbb8483bd9))
- *(checks)* Add ping feature nodefault and add http check - ([8b359b8](https://github.com/PlexSheep/netpulse/commit/8b359b8b460627f4e3e73d344ff2394da4e3f149))
- *(checks)* Do the pinging - ([02961b1](https://github.com/PlexSheep/netpulse/commit/02961b1569935df50ff9f88878df65823a7bd3ae))
- *(ctl)* End now kills after a timeout and removes the pid file if it remains - ([f979aa5](https://github.com/PlexSheep/netpulse/commit/f979aa56bfa538a4f1852dd93c37b829e9061281))
- *(ctl)* Stop the daemon - ([40de014](https://github.com/PlexSheep/netpulse/commit/40de01433e84a80bb0c1d55ea423d14728365dae))
- *(ctl)* Make netpulsed into netpulsectl, a program that starts and stops the daemon - ([b60fcf8](https://github.com/PlexSheep/netpulse/commit/b60fcf8a3e2951de0656177dc52b8febad2276e4))
- *(daemon)* Actually make pings - ([0506658](https://github.com/PlexSheep/netpulse/commit/05066581ee38130cffa50ca523ab482c8e9b4a9c))
- *(daemon)* First daemon app with made up checks - ([5e8f0c4](https://github.com/PlexSheep/netpulse/commit/5e8f0c4a9e36db6e1b54db38ebe250e54ff26a11))
- *(reader)* Analysis of HTTP, analyzis module, outages - ([17c67d4](https://github.com/PlexSheep/netpulse/commit/17c67d46829c47c37c806dba1adb9f633e9c154d))
- *(reader)* Show target in display - ([5960e12](https://github.com/PlexSheep/netpulse/commit/5960e12644ae578b022934764997d4561ed29910))
- *(reader)* Add check tester - ([5c41781](https://github.com/PlexSheep/netpulse/commit/5c41781d27237a00af6cf3e7460314e4b4eddf59))
- *(reader)* Very basic reader - ([e722a6d](https://github.com/PlexSheep/netpulse/commit/e722a6d21412ae1e7f50924e69497ec23e1d90a9))
- *(records)* Add record datastructures for checks - ([e895731](https://github.com/PlexSheep/netpulse/commit/e895731b6adcf013925ad299d903c3478417964d))
- *(store)* Store version checks - ([1f92b17](https://github.com/PlexSheep/netpulse/commit/1f92b177dd05cd69ef6162c45f7580dd6817446c))
- *(store)* [**breaking**] Use zstd compression - ([a926fe0](https://github.com/PlexSheep/netpulse/commit/a926fe0b26562cf1a5d39c8c05341ac5ba88ffbd))
- *(store)* Add_check function - ([3ccc78c](https://github.com/PlexSheep/netpulse/commit/3ccc78cc9f828d8ea879a25f3cf162a5b7e97064))
- *(store)* Store load, create and save logic - ([6c52b10](https://github.com/PlexSheep/netpulse/commit/6c52b10cbc0cd20a02f08a36ddca4ba847c303b0))
- Improved cli, less panics - ([611d1f2](https://github.com/PlexSheep/netpulse/commit/611d1f250ebde533daf0e39106d205df22a785f0))

### 🐛 Bug Fixes

- *(checks)* Http check did not use timeout - ([3f02890](https://github.com/PlexSheep/netpulse/commit/3f02890bfac583bd79caa3bb1b2f1f27db1b9acb))
- *(checks)* Use the latency for icmp - ([9b15d4e](https://github.com/PlexSheep/netpulse/commit/9b15d4efe44bd2ea3780c66dc2d0acce7f335636))
- *(checks)* Add icmpv6 to the all checks - ([07cbf41](https://github.com/PlexSheep/netpulse/commit/07cbf417a975f63263f07729ab0c2d0f53fe17ca))
- *(ctl)* Info was only checking for pidfile, not process - ([b3fa87f](https://github.com/PlexSheep/netpulse/commit/b3fa87fc7df4702242aae562dae054768bb96e61))
- *(daemon)* Daemon did not exit unless the cleanup had an error - ([3535efe](https://github.com/PlexSheep/netpulse/commit/3535efeb302c76b5c15beaa1d6fe5176d4edf480))
- *(daemon)* Daemon high cpu usage because of incorrect flow control - ([6d4505b](https://github.com/PlexSheep/netpulse/commit/6d4505b494fde2ea5043f270b1f216ee5996a866))
- *(http)* Url formatting for ipv6 - ([41dcf94](https://github.com/PlexSheep/netpulse/commit/41dcf944bd9240dd01c40244795447c0c4c06ce0))
- Remove old capabilities code - ([145791a](https://github.com/PlexSheep/netpulse/commit/145791afd00e566ab1b7fab609cf3bdf99f10467))

### 🚜 Refactor

- *(analyze)* Check if checks are totally empty - ([56fe556](https://github.com/PlexSheep/netpulse/commit/56fe5565028086285792218444ef22d06f08b82f))
- *(analyze)* Refactor and generalize analyze outputs - ([09ff8e4](https://github.com/PlexSheep/netpulse/commit/09ff8e4955dee4ee42aed409fa35f9b000c304ea))
- *(analyze)* Own less things - ([8538460](https://github.com/PlexSheep/netpulse/commit/8538460c3ed0c2c1de4930f30726840aaa0ba7d7))
- *(analyze)* Clean up code in analyze - ([3dcd30a](https://github.com/PlexSheep/netpulse/commit/3dcd30aef2838211b53ef9be4e2fee178bd9daee))
- *(daemon)* Mock daemon has failing checks sometimes now - ([05ebf25](https://github.com/PlexSheep/netpulse/commit/05ebf25124d6f8dc52eeb2e801679c6f3c71d93c))
- *(records)* Remove noflags variant and add more trait derives - ([288fe53](https://github.com/PlexSheep/netpulse/commit/288fe53c780ed3681ace3bbe5e9b4135b5179b43))
- *(store)* Do not save on loading an older version - ([b119dc9](https://github.com/PlexSheep/netpulse/commit/b119dc9b74d7bf9ecc2ba2719b848740cbf32a03))
- *(store)* Make create function public - ([42cc405](https://github.com/PlexSheep/netpulse/commit/42cc405272fd98ffe327ff7cc33e0d9ebee46928))
- Use Self::new for version from u8 - ([b9b41f2](https://github.com/PlexSheep/netpulse/commit/b9b41f2abc026323f792bde7a38a48d97f9edc04))
- Use specific targets per check type - ([616e20b](https://github.com/PlexSheep/netpulse/commit/616e20ba5ddc9667cd2ae1a32f271a81a414243f))
- Feature fixes and targets are now always ips - ([4561a3c](https://github.com/PlexSheep/netpulse/commit/4561a3c346525b0125cf58ac3487a29540820dc8))
- Rename ping module to checks - ([c35e907](https://github.com/PlexSheep/netpulse/commit/c35e907d15d35e663fb3255287464858476be035))
- Remove icmp from default_enabled check types, because of CAP_NET_RAW missing from the daemon - ([f7a82b1](https://github.com/PlexSheep/netpulse/commit/f7a82b164e7f92bccba3ed7751ac8a57c618a26a))
- Don't automatically use all check types, just the enabled ones - ([b606eb8](https://github.com/PlexSheep/netpulse/commit/b606eb861fdcc2b4dd82e1de1d5b475efbc60b49))
- Use different error types - ([8d385aa](https://github.com/PlexSheep/netpulse/commit/8d385aacda9237821fefab0d81e6e1bef443344f))

### 📚 Documentation

- *(api)* Dont generate docs for the bins, that conflicts with the lib - ([47db50f](https://github.com/PlexSheep/netpulse/commit/47db50f8d67deee37c5893b13316ed0a503b3eed))
- *(api)* Fix all doc warnings - ([58ea129](https://github.com/PlexSheep/netpulse/commit/58ea1299fcc8b527cda8af13fc4618377745512f))
- *(api)* Analyze module and small fixes - ([2880b68](https://github.com/PlexSheep/netpulse/commit/2880b687fdac16a594e43f1c74d1aa293672315a))
- *(api)* Store module documentations - ([43a48a8](https://github.com/PlexSheep/netpulse/commit/43a48a8a2dda541264674b770bdd766b6b4fec12))
- *(api)* Document the error module - ([a1b42ee](https://github.com/PlexSheep/netpulse/commit/a1b42ee73bbc3ae09c226d805b04b7125897d93d))
- *(api)* Checks with examples and extensive docs - ([f1631b2](https://github.com/PlexSheep/netpulse/commit/f1631b258282374a9d2b5a583b0e47d53a5243e4))
- *(api)* Fix examples - ([9eb4225](https://github.com/PlexSheep/netpulse/commit/9eb4225566e42133f070e4643a7fd4a5018b3cb1))
- *(api)* Tons of api docs with llm help - ([7b42152](https://github.com/PlexSheep/netpulse/commit/7b421523fb1adc98d4fe393d3732681f7f3e3a26))
- Add targets note to readme - ([23dbe11](https://github.com/PlexSheep/netpulse/commit/23dbe119ea589eb5d16606c5d4d4403d651c565f))
- Readme and repo adjustment - ([3880213](https://github.com/PlexSheep/netpulse/commit/388021361a1c0387df8a0655b9fd42986ab0c641))
- Addres CAP_NET_RAW - ([b892c7d](https://github.com/PlexSheep/netpulse/commit/b892c7d687845979d5020cbec71b6cd04f7eda1d))

### ⚙️ Miscellaneous Tasks

- Changelog - ([dc7812e](https://github.com/PlexSheep/netpulse/commit/dc7812ec9225db4c533715098a84266b02077345))
- Set the period_seconds to the production value - ([e2ea8b0](https://github.com/PlexSheep/netpulse/commit/e2ea8b01eff702017429ea97e31d5b1476d888a8))
- Automatic Rust CI changes - ([c28f70a](https://github.com/PlexSheep/netpulse/commit/c28f70a28a79318f4c054760bf54c9e62da90ece))
- Less curl features - ([0a7dd6d](https://github.com/PlexSheep/netpulse/commit/0a7dd6db93ccd80c13dd5f263d8d73c14ae6b1cb))
- Add a build script that adds the caps - ([488cb98](https://github.com/PlexSheep/netpulse/commit/488cb9862b5fd2878fbd293fa767023d3005cf10))
- Script for cap_net_raw - ([ff5c55a](https://github.com/PlexSheep/netpulse/commit/ff5c55a4199cc78084eb046ecc85240027076766))
- Remove unused dependencies - ([9c689c5](https://github.com/PlexSheep/netpulse/commit/9c689c582cd1077f5e764480ce012e3bda68f6af))
- Add deps and rename from base template - ([67cd2fa](https://github.com/PlexSheep/netpulse/commit/67cd2faa613aabb976d09fb707648959572fb424))

