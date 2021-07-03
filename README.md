
# Sofia-SIP

Rust bindings for Sofia-SIP (Alpha stage).


## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
sofia-sip = "*"
```

## Documentation

Sofia-SIP Rust bindings tries to mimic almost as possible the API of Sofia-SIP C library. You can start by learning the concepts of [Sofia SIP User Agent Library - "nua" - High-Level User Agent Module](http://sofia-sip.sourceforge.net/refdocs/nua/).

After this intro, please read [the tests from Nua module](https://github.com/iuridiniz/sofia-sip-sys/blob/main/src/nua/nua_tests.rs).

### Sofia-SIP C docs

Common runtime library:
- [Sofia SIP User Agent Library - "su" - OS Services and Utilities](http://sofia-sip.sourceforge.net/refdocs/su/index.html).
- [Sofia SIP User Agent Library - "sresolv" - Asynchronous DNS Resolver](http://sofia-sip.sourceforge.net/refdocs/sresolv/index.html).
- [Sofia SIP User Agent Library - "ipt" - Utility Module](http://sofia-sip.sourceforge.net/refdocs/ipt/index.html).

SIP Signaling:
- [Sofia SIP User Agent Library - "nua" - High-Level User Agent Module](http://sofia-sip.sourceforge.net/refdocs/nua/).
- [Sofia SIP User Agent Library - "nea" - SIP Events Module](http://sofia-sip.sourceforge.net/refdocs/nea/index.html).
- [Sofia SIP User Agent Library - "iptsec" - Authentication Module](http://sofia-sip.sourceforge.net/refdocs/iptsec/index.html).
- [Sofia SIP User Agent Library - "nta" - SIP Transactions Module](http://sofia-sip.sourceforge.net/refdocs/nta/index.html).
- [Sofia SIP User Agent Library - "tport" - Transport Module](http://sofia-sip.sourceforge.net/refdocs/tport/index.html).
- [Sofia SIP User Agent Library - "sip" - SIP Parser Module](http://sofia-sip.sourceforge.net/refdocs/sip/index.html).
- [Sofia SIP User Agent Library - "msg" - Message Parser Module](http://sofia-sip.sourceforge.net/refdocs/msg/index.html).
- [Sofia SIP User Agent Library - "url" - URL Module](http://sofia-sip.sourceforge.net/refdocs/url/index.html).
- [Sofia SIP User Agent Library - "bnf" - String Parser Module](http://sofia-sip.sourceforge.net/refdocs/bnf/index.html).

HTTP subsystem:
- [Sofia SIP User Agent Library - "nth" - HTTP Transactions Module](http://sofia-sip.sourceforge.net/refdocs/nth/index.html).
- [Sofia SIP User Agent Library - "http" - HTTP Parser Module](http://sofia-sip.sourceforge.net/refdocs/http/index.html).

SDP processing:
- [Sofia SIP User Agent Library - "soa" - SDP Offer/Answer Engine Module](http://sofia-sip.sourceforge.net/refdocs/soa/index.html).
- [Sofia SIP User Agent Library - "sdp" - SDP Module](http://sofia-sip.sourceforge.net/refdocs/sdp/index.html).

Other:
- [Sofia SIP User Agent Library - "features" Module](http://sofia-sip.sourceforge.net/refdocs/features/index.html).
- [Sofia SIP User Agent Library - "stun" - STUN Client and Server Module](http://sofia-sip.sourceforge.net/refdocs/stun/index.html).

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
    - NUA: Basic support to send and receive SIP MESSAGE's, allowing to create a chat using SIP.

- Version 0.2.0
    - NUA: Basic support to send SIP INVITE(SDP)/REGISTER(auth) and receive SIP INVITE(SDP), allowing to create a simple soft phone.
    - Others modules: basic support to make NUA objectives work.

- Version 0.3.0
    - NUA: Support receive SIP REGISTER(auth), allowing to create a simple SIP PBX.
    - Others modules: basic support to make NUA objectives work.

- Version 1.0.0
    - NUA: Full bindings for NUA.
    - SDP: Full support for SDP parsing.

NUA is the High-Level User Agent Module of lib-sofia. To learn more about sofia modules, go to [reference documentation for libsofia-sip-ua submodules](http://sofia-sip.sourceforge.net/refdocs/building.html).


