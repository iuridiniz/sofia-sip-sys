use sofia_sip::su;
use sofia_sip::Handle;
use sofia_sip::Nua;
use sofia_sip::NuaEvent;
use sofia_sip::Root;
use sofia_sip::Sip;
use sofia_sip::Tag;
use sofia_sip::TagBuilder;

use adorn::adorn;
use serial_test::serial;

use std::cell::RefCell;
use std::rc::Rc;

fn wrap(f: fn()) {
    /* manual deinit (because tests do not run atexit) */
    if let Err(e) = std::panic::catch_unwind(|| {
        su::init().unwrap();
        su::init_default_root().unwrap();
        f();
        su::deinit_default_root();
        su::deinit();
    }) {
        su::deinit_default_root();
        su::deinit();
        println!(
            "******************************************************\n\
             PANIC INSIDE WRAPPER\n\
             `#[adorn(wrap)]` may give a wrong line that panicked\n\
             ******************************************************\n"
        );
        std::panic::resume_unwind(e);
    }
}

#[test]
#[adorn(wrap)]
#[serial]
fn test_case_nua_send_message_to_itself() {
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
    let my_message = String::from("Hi\n");
    let root = Root::create().unwrap();
    let url = String::from("sip:127.0.0.1:9997");

    let mut nua = {
        let url = Tag::NuUrl(url.clone());
        let tags = TagBuilder::default().tag(url).collect();
        Nua::create_with_root(&root, &tags).unwrap()
    };

    let recv_message = Rc::new(RefCell::new(String::new()));

    {
        let recv_message = recv_message.clone();
        nua.callback(
            move |nua: &mut Nua,
                  event: NuaEvent,
                  status: u32,
                  phrase: String,
                  handle: Option<&Handle>,
                  sip: Sip,
                  tags: Vec<Tag>| {
                dbg!(&nua, &event, &status, &phrase, &handle, &sip, &tags);
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
                        let payload = sip.payload().as_utf8_lossy();
                        println!("Payload: {:?}", &payload);
                        recv_message.borrow_mut().push_str(&payload);
                    }
                    NuaEvent::ReplyMessage => {}
                    _ => {}
                }
            },
        );
    }

    let handle = {
        let tags = TagBuilder::default()
            .tag(Tag::SipToStr(url.clone()))
            .tag(Tag::NuUrl(url.clone()))
            .collect();
        Handle::create(&nua, &tags).unwrap()
    };

    let tags = TagBuilder::default()
        .tag(Tag::SipSubjectStr("NUA".into()))
        .tag(Tag::SipToStr(url.clone()))
        .tag(Tag::NuUrl(url.clone()))
        .tag(Tag::SipContentTypeStr("text/plain".into()))
        .tag(Tag::SipPayloadStr(my_message.clone()))
        .collect();

    handle.message(&tags);
    root.sleep(0);

    assert_eq!(&*recv_message.borrow(), &my_message);
}

#[test]
#[adorn(wrap)]
#[serial]
fn test_case_nua_a_send_message_to_nua_b() {
    /* see <lib-sofia-ua-c>/tests/test_simple.c::test_message */
    /*
    A                    B
    |-------MESSAGE----->|
    |<--------200--------| (method allowed, responded)
    |                    |

                           ______(NETWORK)_____
                          /                    \
    A                 NUA STACK (A)         NUA STACK (B)             B
    |                     |                     |                     |
    |    nua::handle(B)   |                     |                     |
    |-------------------->|                     |                     |
    |                     |                     |                     |
    |  handle::message()  |                     |                     |
    |------------------->[_]      [MESSAGE]     |                     |
    |                    [_]------------------>[_]   IncomingMessage  |
    |                    [_]                   [_]------------------->|
    |                    [_]                   [_]   nua::handle(A)   |
    |                    [_]      [200 OK]     [_]                    |
    |    ReplyMessage    [_]<------------------[_]                    |
    |<------------------ [_]                    |                     |
    |                     |                     |                     |
    |                     |                     |                     |

    */
    let nua_a_url = String::from("sip:127.0.0.1:5080");
    let mut nua_a = {
        let url = Tag::NuUrl(nua_a_url.clone());
        let tags = TagBuilder::default().tag(url).collect();
        Nua::create(&tags).unwrap()
    };
    let nua_b_url = String::from("sip:127.0.0.1:5081");
    let mut nua_b = {
        let url = Tag::NuUrl(nua_b_url.clone());
        let tags = TagBuilder::default().tag(url).collect();
        Nua::create(&tags).unwrap()
    };
    let recv_message = Rc::new(RefCell::new(String::new()));
    {
        /* NUA B */
        let recv_message = recv_message.clone();
        nua_b.callback(
            move |nua: &mut Nua,
                  event: NuaEvent,
                  status: u32,
                  phrase: String,
                  handle: Option<&Handle>,
                  sip: Sip,
                  tags: Vec<Tag>| {
                // dbg!(&nua, &event, &status, &phrase, &handle, &sip, &tags);
                let root: &Root = nua.root();
                println!("[NUA _B]Event: {:?}", &event);
                match event {
                    NuaEvent::ReplyShutdown => {}
                    NuaEvent::IncomingMessage => {
                        println!("[NUA _B]Received MESSAGE: {} {}", status, &phrase);
                        println!("[NUA _B]From: {}", sip.from());
                        println!("[NUA _B]To: {}", sip.to());
                        println!("[NUA _B]Subject: {}", sip.subject());
                        println!("[NUA _B]ContentType: {}", sip.content_type());
                        let payload = sip.payload().as_utf8_lossy();
                        println!("[NUA _B]Payload: {:?}", &payload);
                        recv_message.borrow_mut().push_str(&payload);
                    }
                    NuaEvent::ReplyMessage => {}
                    _ => {}
                }
            },
        );
    }
    {
        /* NUA A */
        let recv_message = recv_message.clone();
        nua_a.callback(
            move |nua: &mut Nua,
                  event: NuaEvent,
                  status: u32,
                  phrase: String,
                  handle: Option<&Handle>,
                  sip: Sip,
                  tags: Vec<Tag>| {
                // dbg!(&nua, &event, &status, &phrase, &handle, &sip, &tags);
                let root: &Root = nua.root();
                println!("[NUA A_]Event: {:?}", &event);
                match event {
                    NuaEvent::ReplyShutdown => {}
                    NuaEvent::IncomingMessage => {}
                    NuaEvent::ReplyMessage => {
                        root.break_();
                    }
                    _ => {}
                }
            },
        );
    }
    let my_message = String::from("Hi Sofia SIP\n");

    let handle = {
        let tags = TagBuilder::default()
            .tag(Tag::SipToStr(nua_b_url.clone()))
            .tag(Tag::NuUrl(nua_b_url.clone()))
            .collect();
        Handle::create(&nua_a, &tags).unwrap()
    };

    let tags = TagBuilder::default()
        .tag(Tag::SipSubjectStr("NUA".into()))
        .tag(Tag::SipToStr(nua_b_url.clone()))
        .tag(Tag::NuUrl(nua_b_url.clone()))
        .tag(Tag::SipContentTypeStr("text/plain".into()))
        .tag(Tag::SipPayloadStr(my_message.clone()))
        .collect();

    handle.message(&tags);
    println!("--> Root run start");
    Root::get_default_root().unwrap().run();
    // Root::get_default_root().unwrap().sleep(0);
    println!("--> Root run end");

    assert_eq!(&*recv_message.borrow(), &my_message);
}
