//! The remember foot: the arrows walking the reaches, and the path reach, offered from the link,
//! stepped a segment at a time or typed over.

use linkunbound_core::{
    Strings, first_segment_path, path_scope, step_path, under_path, whole_path,
};
use slint::{ComponentHandle, Model};

use crate::{Picker, Reaches, Said};

/// The row's entry for a link: its host and first segment, dead without one or behind a wrapper.
#[must_use]
pub fn said(words: &Strings, url: &str, wrapped: bool) -> Said {
    let first = first_segment_path(url);
    Said {
        label: words.reach_path.to_owned(),
        full: Strings::fill(words.remember_path, first.as_deref().unwrap_or_default()),
        keeps: true,
        dead: wrapped || first.is_none(),
    }
}

/// The foot for a new link: the first segment offered, the whole path as far as + reaches.
pub fn dress(window: &Picker, words: &Strings, url: &str) {
    window.set_path_value(first_segment_path(url).unwrap_or_default().into());
    window.set_path_whole(whole_path(url).unwrap_or_default().into());
    window.set_path_editing(false);
    window.set_path_bad(false);
    window.set_path_edit_label(words.reach_edit.into());
    window.set_path_refusal(words.path_not_path.into());
}

/// What the foot answers while it is used. A typed path must cover the link, or the rule it
/// writes would never answer the link it was written for.
pub fn wire(window: &Picker) {
    let handle = window.as_weak();
    window.on_step_reach(move |delta| {
        let Some(window) = handle.upgrade() else {
            return 0;
        };
        let dead: Vec<bool> = window.get_all_reaches().iter().map(|r| r.dead).collect();
        next_live(&dead, window.get_reach_index(), delta)
    });
    window.on_path_check(|said, whole| {
        path_scope(&said)
            .filter(|path| under_path(&format!("https://{whole}"), path))
            .unwrap_or_default()
            .into()
    });
    window
        .on_path_step(|at, whole, delta| step_path(&at, &whole, delta).unwrap_or_default().into());
}

/// For the path reach, the path the foot holds, tidied, when it covers `url`: `None` refuses,
/// with the foot marked and the reason shown. Every other reach passes with no path.
#[must_use]
pub fn kept(window: &Picker, reach: Reaches, url: &str) -> Option<Option<String>> {
    if reach != Reaches::Path {
        return Some(None);
    }
    let path = path_scope(&window.get_path_value()).filter(|path| under_path(url, path));
    if path.is_none() {
        window.set_path_bad(true);
        window.set_alarming(true);
        window.set_problem(window.get_path_refusal());
    } else if window.get_path_bad() {
        window.set_path_bad(false);
        window.set_problem(Default::default());
    }
    path.map(Some)
}

/// Where the keys land next, stepping over the reaches that are dead for this link: only the
/// pointer was kept off them, and a rule written on one covered every site the wrapper serves.
/// Stays put when nothing live lies that way.
#[must_use]
pub fn next_live(dead: &[bool], from: i32, delta: i32) -> i32 {
    let mut at = from;
    loop {
        at += delta;
        let Some(is_dead) = usize::try_from(at).ok().and_then(|i| dead.get(i)) else {
            return from;
        };
        if !is_dead {
            return at;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{dress, kept, next_live, said};
    use crate::Reaches;
    use linkunbound_core::{Language, Scope};

    const LINK: &str = "https://vercel.com/trozl/m4500-twin/settings";

    #[test]
    fn the_path_reach_offers_the_first_segment_and_is_dead_without_one() {
        let words = Language::English.strings();
        let offered = said(&words, LINK, false);
        assert_eq!(offered.full, "Remember everything under vercel.com/trozl");
        assert!(!offered.dead);
        assert!(
            said(&words, LINK, true).dead,
            "a wrapper's path is not the link's"
        );
        assert!(said(&words, "https://vercel.com/", false).dead);
        assert_eq!(
            Reaches::Path.scope(LINK, "vercel.com"),
            Some(Scope::Path("vercel.com/trozl".to_owned()))
        );
    }

    #[test]
    fn a_path_opens_only_when_it_covers_the_link() {
        i_slint_backend_testing::init_no_event_loop();
        let window = crate::Picker::new().expect("a window");
        let words = Language::English.strings();
        dress(&window, &words, LINK);
        assert_eq!(
            window.get_path_whole(),
            "vercel.com/trozl/m4500-twin/settings"
        );
        assert_eq!(kept(&window, Reaches::Site, LINK), Some(None));
        assert_eq!(
            kept(&window, Reaches::Path, LINK),
            Some(Some("vercel.com/trozl".to_owned()))
        );
        window.set_path_value("Vercel.com/Trozl/M4500-twin".into());
        assert_eq!(
            kept(&window, Reaches::Path, LINK),
            Some(Some("vercel.com/trozl/m4500-twin".to_owned()))
        );
        assert!(!window.get_path_bad());

        window.set_path_value("github.com/foo".into());
        assert_eq!(
            kept(&window, Reaches::Path, LINK),
            None,
            "it misses this link"
        );
        assert!(window.get_path_bad());
        assert_eq!(window.get_problem(), words.path_not_path);

        window.set_path_value("vercel.com/trozl/".into());
        assert!(kept(&window, Reaches::Path, LINK).is_some());
        assert!(!window.get_path_bad(), "a mended path clears the refusal");
        assert_eq!(window.get_problem(), "");

        dress(&window, &words, LINK);
        window.set_path_value("vercel.com".into());
        assert_eq!(
            kept(&window, Reaches::Path, LINK),
            None,
            "a host is not a path"
        );
    }

    #[test]
    fn the_foot_checks_and_steps_the_path_against_the_link() {
        i_slint_backend_testing::init_no_event_loop();
        let window = crate::Picker::new().expect("a window");
        super::wire(&window);
        let whole = "vercel.com/trozl/m4500-twin".into();
        assert_eq!(
            window.invoke_path_check("VERCEL.com/trozl/".into(), whole),
            "vercel.com/trozl"
        );
        let whole = "vercel.com/trozl/m4500-twin".into();
        assert_eq!(
            window.invoke_path_check("github.com/trozl".into(), whole),
            ""
        );
        let whole = "vercel.com/trozl/m4500-twin".into();
        assert_eq!(
            window.invoke_path_step("vercel.com/trozl".into(), whole, 1),
            "vercel.com/trozl/m4500-twin"
        );
    }

    /// A wrapper nobody could unwrap leaves three reaches dead; the arrows used to land on them
    /// anyway, and «1» then wrote a rule for everything the wrapper serves.
    #[test]
    fn the_keys_step_over_the_dead_reaches_and_stay_put_at_the_ends() {
        let wrapped = [false, true, true, true, false];
        assert_eq!(next_live(&wrapped, 0, 1), 4, "straight to the origin reach");
        assert_eq!(next_live(&wrapped, 4, -1), 0);
        assert_eq!(next_live(&wrapped, 4, 1), 4, "nothing beyond");
        assert_eq!(next_live(&wrapped, 0, -1), 0);
        let no_origin = [false, true, true, true];
        assert_eq!(next_live(&no_origin, 0, 1), 0, "every other reach is dead");
        let plain = [false, false, true, false];
        assert_eq!(next_live(&plain, 1, 1), 3, "the same-site reach is skipped");
        assert_eq!(next_live(&plain, 3, -1), 1);
        assert_eq!(next_live(&[], 0, 1), 0);
    }
}
