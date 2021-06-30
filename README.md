
# Sofia-SIP

Rust bindings for Sofia-SIP (Alpha stage).


## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
sofia-sip = "*"
```
## Acknowledgements

 - [Original Sofia-SIP (not currently maintained)](http://sofia-sip.sourceforge.net/).
 - [Freeswitch version of Sofia-SIP (Currently maintained)](https://bulldogjob.com/news/449-how-to-write-a-good-readme-for-your-github-project).

  
## Authors

- [@iuridiniz](https://www.github.com/iuridiniz)

  
## License

- Rust bindings: [MIT](https://choosealicense.com/licenses/mit/)
- Sofia-SIP C library: [LGPL-2.1-or-later](https://choosealicense.com/licenses/lgpl-2.1/)

Before compiling statically, please read [this](https://www.gnu.org/licenses/gpl-faq.html#LGPLStaticVsDynamic).
## Roadmap
- Version 0.1.0
    - NUA: Basic support to send SIP INVITE(SDP)/REGISTER(auth) and receive SIP INVITE(SDP), allowing to create a simple soft phone.
    - Others modules: basic support to make NUA objectives work.

- Version 0.2.0
    - NUA: Support receive SIP REGISTER(auth), allowing to create a simple SIP PBX.
    - Others modules: basic support to make NUA objectives work.


- Version 1.0.0
    - NUA: Full bindings for NUA.
    - SDP: Full support for SDP parsing.

NUA is the High-Level User Agent Module of lib-sofia. To learn more about sofia modules, go to [reference documentation for libsofia-sip-ua submodules](http://sofia-sip.sourceforge.net/refdocs/building.html).

## Documentation

Sofia-SIP Rust bindings tries to mimic almost as possible the API of Sofia-SIP C library. You can start by learn the concepts of [Sofia SIP User Agent Library - "nua" - High-Level User Agent Module
](http://sofia-sip.sourceforge.net/refdocs/nua/).

After this intro, please read [the tests from Nua module](https://github.com/iuridiniz/sofia-sip-sys/blob/main/src/nua/nua.rs).
