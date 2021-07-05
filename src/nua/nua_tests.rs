use crate::Handle;
use crate::Nua;
use crate::NuaEvent;
use crate::Root;
use crate::Sip;
use crate::Tag;
use crate::TagBuilder;

use crate::su::wrap;
use adorn::adorn;
use serial_test::serial;

#[test]
#[adorn(wrap)]
#[serial]
fn create_nua_with_default_root() {
    let b = TagBuilder::default();

    b.create().unwrap();
}

#[test]
#[adorn(wrap)]
#[serial]
fn create_nua_with_custom_root() {
    let root = Root::new().unwrap();

    let b = TagBuilder::default();
    let b = b.root(&root);

    b.create().unwrap();
}

#[test]
#[adorn(wrap)]
#[serial]
fn create_nua_with_custom_url() {
    let url = Tag::NuUrl("sip:*:5080").unwrap();

    let root = Root::new().unwrap();

    let b = TagBuilder::default();
    let b = b.root(&root);
    let b = b.tag(url);

    b.create().unwrap();
}

#[test]
#[adorn(wrap)]
#[serial]
fn create_two_nua_with_same_port() {
    let url = Tag::NuUrl("sip:*:5080").unwrap();

    let root = Root::new().unwrap();

    let b = TagBuilder::default();
    let b = b.root(&root);
    let b = b.tag(url);

    let _nua_a = b.create().unwrap();

    let url = Tag::NuUrl("sip:*:5080").unwrap();

    let root = Root::new().unwrap();

    let b = TagBuilder::default();
    let b = b.root(&root);
    let b = b.tag(url);

    assert!(b.create().is_err());
}

// #[test]
// #[ignore]
// #[adorn(wrap)]
// #[serial]
// fn test_nua_a_send_message_to_nua_b() {
//     /* see <lib-sofia-ua-c>/tests/test_simple.c::test_message */
//     /*
//     A                    B
//     |-------MESSAGE----->|
//     |<--------200--------| (method allowed, responded)
//     |                    |

//                            ______(NETWORK)_____
//                           /                    \
//     A                 NUA STACK (A)         NUA STACK (B)             B
//     |                     |                     |                     |
//     |    nua::handle(B)   |                     |                     |
//     |-------------------->|                     |                     |
//     |                     |                     |                     |
//     |  handle::message()  |                     |                     |
//     |------------------->[_]      [MESSAGE]     |                     |
//     |                    [_]------------------>[_]   IncomingMessage  |
//     |                    [_]                   [_]------------------->|
//     |                    [_]                   [_]   nua::handle(A)   |
//     |                    [_]      [200 OK]     [_]                    |
//     |    ReplyMessage    [_]<------------------[_]                    |
//     |<------------------ [_]                    |                     |
//     |                     |                     |                     |
//     |                     |                     |                     |

//     */
//     let root = Root::new().unwrap();

//     let mut nua_a = {
//         let url = Tag::NuUrl("sip:127.0.0.1:5080").unwrap();
//         TagBuilder::default().root(&root).tag(url).create().unwrap()
//     };

//     // let nua_b = {
//     //     let url = Tag::NuUrl("sip:127.0.0.1:5081").unwrap();
//     //     TagBuilder::default()
//     //         .root(&root)
//     //         .tag(url)
//     //         .create().unwrap()
//     // };

//     nua_a.callback(|nua: &mut Nua, event: NuaEvent, status: u32, phrase: String| {
//         dbg!(&nua, &event, &status, &phrase);
//     });

//     let url = "Joe User <sip:joe.user@localhost:5081;param=1>;tag=12345678";

//     let handle = TagBuilder::default()
//         // .tag(Tag::SipTo(url.clone()).unwrap())
//         // .tag(Tag::NuUrl(url.clone()).unwrap())
//         .create_handle(&nua_a)
//         .unwrap();

//     // dbg!(&handle);

//     let tags = TagBuilder::default()
//         .tag(Tag::SipSubject("NUA").unwrap())
//         .tag(Tag::SipTo(url.clone()).unwrap())
//         .tag(Tag::NuUrl(url.clone()).unwrap())
//         .tag(Tag::SipContentType("text/plain").unwrap())
//         .tag(Tag::SipPayload("Hi\n").unwrap())
//         .create_tags();
//     // dbg!(&tags);

//     println!("BEFORE MESSAGE");
//     handle.message(tags);
//     println!("AFTER MESSAGE");

//     root.sleep(100);
//     println!("AFTER RUN");

//     panic!("abort");
// }

#[test]
// #[ignore]
#[adorn(wrap)]
#[serial]
fn nua_send_message_to_itself() {
    /* see <lib-sofia-ua-c>/tests/test_simple.c::test_message */
    /*

    A
    |-------------------\
    |<------MESSAGE-----/
    |
    |-------------------\
    |<--------200-------/
    |

                           _(LOOPBACK NETWORK)_
                          /                    \
    A                 NUA STACK (A)
    |                     |
    |   nua::handle(A')   |
    |-------------------->|
    |                     |
    |  handle::message()  |
    |------------------->[_]     [MESSAGE]
    |                    [_]-------------------\
    |                    [_]                   |
    |                    [_]                   |
    |  IncomingMessage   [_]<------------------/
    |<-------------------[_]
    |   nua::handle(A")  [_]
    |                    [_]      [200 OK]
    |                    [_]-------------------\
    |                    [_]                   |
    |    ReplyMessage    [_]<------------------/
    |<------------------ [_]
    |                    [_]
    | handle::destroy()  [_]
    |------------------->[_]
    |                     |
    */
    let my_message = "Hi\n";
    let root = Root::new().unwrap();
    let url = std::rc::Rc::new("sip:127.0.0.1:9997");

    let mut nua = {
        let url = Tag::NuUrl(&url.clone()).unwrap();
        TagBuilder::default().root(&root).tag(url).create().unwrap()
    };
    {
        // let my_message = my_message.clone();
        nua.callback(
            |nua: &mut Nua,
             event: NuaEvent,
             status: u32,
             phrase: String,
             handle: Option<&Handle>,
             sip: Sip,
             tags: Vec<Tag>| {
                dbg!(&nua, &event, &status, &phrase, &handle, &sip, &tags);
                // dbg!(&event);
                let root: &Root = nua.root();
                match event {
                    NuaEvent::ReplyShutdown => {
                        root.break_();
                    }
                    NuaEvent::IncomingMessage => {
                        println!("Received MESSAGE: {} {}", status, &phrase);
                        println!("From: {}", sip.from());
                        println!("To: {}", sip.to());
                        println!("Subject: {}", sip.subject());
                        println!("ContentType: {}", sip.content_type());
                        println!("Payload: {:?}", sip.payload().as_utf8_lossy());
                        assert_eq!(sip.payload().as_utf8_lossy(), my_message);
                    }
                    NuaEvent::ReplyMessage => {
                        // dbg!(my_message);
                    }
                    _ => {}
                }
            },
        );
    }

    let handle = TagBuilder::default()
        .tag(Tag::SipTo(&url.clone()).unwrap())
        .tag(Tag::NuUrl(&url.clone()).unwrap())
        .create_handle(&nua)
        .unwrap();

    // dbg!(&handle);

    let tags = TagBuilder::default()
        .tag(Tag::SipSubject("NUA").unwrap())
        .tag(Tag::SipTo(&url.clone()).unwrap())
        .tag(Tag::NuUrl(&url.clone()).unwrap())
        .tag(Tag::SipContentType("text/plain").unwrap())
        .tag(Tag::SipPayloadString(my_message).unwrap())
        .create_tags();

    handle.message(tags);
    root.sleep(1000);

    panic!("*********************** ABORTED ***********************");
}
