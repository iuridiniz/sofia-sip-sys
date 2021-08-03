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
fn test_case_basic_call_incomplete() {
    // see <lib-sofia-ua-c>/tests/test_basic_call.c::test_basic_call_1
    // A                    B
    // |-------INVITE------>|
    // |<----100 Trying-----|
    // |                    |
    // |<----180 Ringing----|
    // |                    |
    // |<------200 OK-------|
    // |--------ACK-------->|
    // |                    |
    // |<-------BYE---------|
    // |-------200 OK------>|
    // |                    |

    //                        ______(NETWORK)_____
    //                       /                    \
    // A                 NUA STACK (A)         NUA STACK (B)             B
    // |                     |                     |                     |
    // |   nua::handle(B)    |                     |                     |
    // |-------------------->|                     |                     |
    // |                     |                     |                     |
    // |  handle::invite()   |                     |                     |
    // |------------------->[_]    [INVITE/SDP]    |                     |
    // |                    [_]------------------>[_]   IncomingInvite   |
    // |                    [_]                   [_]------------------->|
    // |                    [_]                   [_]   nua::handle(A)   |
    // |                    [_]                   [_]                    |
    // |                    [_]    [100 Trying]   [_]                    |
    // |                    [_]<------------------[_]                    |
    // |                    [_]   [180 Ringing]   [_]                    |
    // |                    [_]<------------------[_]                    |
    // |                    [_]                   [_]  handle::respond() |
    // |                    [_]      [200 OK]     [_]<-------------------|
    // |                    [_]<------------------[_]                    |
    // |     ReplyInvite    [_]                   [_]                    |
    // |<-------------------[_]       [ACK]       [_]                    |
    // |                    [_]------------------>[_]   IncomingActive   |
    // |   IncomingActive   [_]                   [_]------------------->|
    // |<-------------------[_]                   [_]                    |
    // |                    [_]                   [_]    handle::bye()   |
    // |                    [_]       [BYE]       [_]<-------------------|
    // |    IncomingBye     [_]<------------------[_]                    |
    // |<-------------------[_]      [200 OK]     [_]                    |
    // |                    [_]------------------>[_]                    |
    // |                     |                     |                     |
    // |                     |                     |                     |
    let nua_a_url = "sip:127.0.0.1:5080";
    let mut nua_a = {
        let url = Tag::NuUrl(nua_a_url);
        let tags = TagBuilder::default().tag(url).collect();
        Nua::create(&tags).unwrap()
    };
    let nua_b_url = "sip:127.0.0.1:5081";
    let mut nua_b = {
        let url = Tag::NuUrl(nua_b_url);
        let tags = TagBuilder::default().tag(url).collect();
        Nua::create(&tags).unwrap()
    };

    {
        /* NUA B */
        nua_b.callback(
            |nua: &mut Nua,
             event: NuaEvent,
             status: u32,
             phrase: String,
             handle: Option<&Handle>,
             sip: Sip,
             tags: Vec<Tag>| {
                // dbg!(&nua, &event, &status, &phrase, &handle, &sip, &tags);
                println!(
                    "[NUA _B]Event: {:?} // status: {:?} // phrase: {:?}",
                    &event, &status, &phrase
                );
                match event {
                    _ => {}
                }
            },
        );
    }

    {
        /* NUA A */
        nua_a.callback(
            |nua: &mut Nua,
             event: NuaEvent,
             status: u32,
             phrase: String,
             handle: Option<&Handle>,
             sip: Sip,
             tags: Vec<Tag>| {
                // dbg!(&nua, &event, &status, &phrase, &handle, &sip, &tags);
                println!(
                    "[NUA A_]Event: {:?} // status: {:?} // phrase: {:?}",
                    &event, &status, &phrase
                );
                match event {
                    _ => {}
                }
            },
        );
    }

    let handle = {
        let tags = TagBuilder::default()
            .tag(Tag::SipToStr(nua_b_url))
            .tag(Tag::NuUrl(nua_b_url))
            .collect();
        Handle::create(&nua_a, &tags).unwrap()
    };
    dbg!(&handle);

    let tags = TagBuilder::default()
        .tag(Tag::NuUrl(nua_b_url))
        .tag(Tag::SoaUserSdpStr("m=audio 5008 RTP/AVP 8"))
        .tag(Tag::NuMUsername("a+a"))
        .tag(Tag::NuMDisplay("Alice"))
        .collect();

    handle.invite(&tags);

    Root::get_default_root().unwrap().step0();

    println!("--> Test end");
    // assert!(false);
}
