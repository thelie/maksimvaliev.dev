use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::{
    content::{Build, Content, Project},
    lang::Lang,
};

const STYLE: &str = include_str!("style.css");
const FAVICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='8' fill='%231d1d1f'/%3E%3Ctext x='16' y='22' font-family='-apple-system,Helvetica,Arial' font-size='16' font-weight='600' fill='%23f5f5f7' text-anchor='middle'%3EMV%3C/text%3E%3C/svg%3E";

fn layout(lang: Lang, title: &str, description: &str, head: Markup, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang=(lang.code()) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="description" content=(description);
                meta name="color-scheme" content="light dark";
                meta name="theme-color" content="#fbfbfd" media="(prefers-color-scheme: light)";
                meta name="theme-color" content="#000000" media="(prefers-color-scheme: dark)";
                meta property="og:title" content=(title);
                meta property="og:description" content=(description);
                meta property="og:locale" content=(lang.locale());
                link rel="icon" href=(FAVICON);
                (head)
                title { (title) }
                style { (PreEscaped(STYLE)) }
            }
            body { (body) }
        }
    }
}

fn lang_switch(current: Lang) -> Markup {
    html! {
        div.lang role="group" aria-label="Language / Язык" {
            @for l in Lang::ALL {
                a href=(l.path()) hreflang=(l.code()) lang=(l.code())
                    aria-current=[(l == current).then_some("page")] { (l.label()) }
            }
        }
    }
}

pub fn index(c: &Content, lang: Lang, build: &Build) -> String {
    let ui = &c.ui;
    let lines: Vec<&str> = c.headline.lines().collect();
    let (featured, rest) = c.projects.split_first().map_or((None, &[][..]), |(f, r)| (Some(f), r));

    let head = html! {
        link rel="canonical" href={ (c.url) (lang.path()) };
        @for l in Lang::ALL {
            link rel="alternate" hreflang=(l.code()) href={ (c.url) (l.path()) };
        }
        link rel="alternate" hreflang="x-default" href={ (c.url) "/" };
    };

    let body = html! {
        nav.globalnav {
            div.wrap {
                a.brand href="#top" { (c.name) }
                div.right {
                    ul {
                        li { a href="#about" { (ui.nav_about) } }
                        li { a href="#work" { (ui.nav_work) } }
                        @if let Some(pr) = &c.prices { li { a href="#prices" { (pr.nav) } } }
                        li { a href="#resume" { (ui.nav_resume) } }
                        li { a href="#contact" { (ui.nav_contact) } }
                    }
                    (lang_switch(lang))
                }
            }
        }

        main {
            header.hero #top {
                p.eyebrow { (c.role) }
                h1 {
                    @for (i, line) in lines.iter().enumerate() {
                        @if i > 0 { br; }
                        span class=[(i + 1 == lines.len() && i > 0).then_some("grad")] { (line) }
                    }
                }
                p.intro { (c.intro) }
                div.cta {
                    a.btn href="#work" { (ui.see_work) }
                    a.more href="#contact" { (ui.get_in_touch) }
                }
            }

            section.section.alt #about {
                div.wrap {
                    h2.reveal { (c.about.title) }
                    p.lead.reveal { (c.about.lead) }
                    div.grid {
                        @for item in &c.about.items {
                            div.tile.reveal {
                                span.symbol aria-hidden="true" { (item.symbol) }
                                h3 { (item.title) }
                                p { (item.text) }
                            }
                        }
                    }
                }
            }

            section.section #work {
                div.wrap {
                    h2.reveal { (ui.work_title) }
                    p.lead.reveal { (ui.work_lead) }
                    div.projects {
                        @if let Some(p) = featured {
                            (project(p, true, &ui.learn_more))
                        }
                        @for p in rest {
                            (project(p, false, &ui.learn_more))
                        }
                    }
                }
            }

            @if let Some(pr) = &c.prices {
                section.section.alt #prices {
                    div.wrap {
                        h2.reveal { (pr.title) }
                        p.lead.reveal { (pr.lead) }
                        div.grid {
                            @for item in &pr.items {
                                div.tile.price.reveal {
                                    h3 { (item.name) }
                                    p.amount { (item.price) }
                                    p.meta { (item.term) }
                                    p { (item.text) }
                                }
                            }
                        }
                        @if !pr.included.is_empty() {
                            div.card.included.reveal {
                                h3 { (pr.included_title) }
                                ul.chips { @for i in &pr.included { li { (i) } } }
                                p.meta { (pr.note) }
                            }
                        }
                    }
                }
            }

            section class={ "section" @if c.prices.is_none() { " alt" } } #resume {
                div.wrap {
                    h2.reveal { (ui.resume_title) }
                    div.resume {
                        div.jobs {
                            @for job in &c.experience {
                                article.job.reveal {
                                    p.meta {
                                        (job.period)
                                        @if let Some(loc) = &job.location { " · " (loc) }
                                    }
                                    h3 { (job.role) }
                                    p.company { (job.company) }
                                    @if !job.highlights.is_empty() {
                                        ul {
                                            @for h in &job.highlights { li { (h) } }
                                        }
                                    }
                                }
                            }
                        }
                        aside.side {
                            @if !c.skills.is_empty() {
                                div.card.reveal {
                                    h3 { (ui.skills) }
                                    @for g in &c.skills {
                                        p.meta { (g.group) }
                                        ul.chips {
                                            @for s in &g.items { li { (s) } }
                                        }
                                    }
                                }
                            }
                            @if !c.education.is_empty() {
                                div.card.reveal {
                                    h3 { (ui.education) }
                                    @for e in &c.education {
                                        p.degree { (e.degree) }
                                        p.meta { (e.school) " · " (e.period) }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            section class={ "section contact" @if c.prices.is_some() { " alt" } } #contact {
                div.wrap {
                    h2.reveal { (ui.contact_title) }
                    p.lead.reveal { (ui.contact_lead) }
                    div.cta.reveal {
                        a.btn href={ "mailto:" (c.email) } { (c.email) }
                    }
                    @if !c.links.is_empty() {
                        ul.social.reveal {
                            @for l in &c.links {
                                li { a.more href=(l.url) rel="me noopener" { (l.label) } }
                            }
                        }
                    }
                }
            }
        }

        footer {
            div.wrap {
                p { "© " (c.name) " · " (c.location) }
                p {
                    (ui.footer_note) " · "
                    a href="/status" { "v" (build.version) " (" (build.commit) ")" } " · "
                    a href="/source" { (ui.source) }
                }
            }
        }
    };

    layout(lang, &format!("{} — {}", c.name, c.role), &c.intro, head, body).into_string()
}

fn project(p: &Project, featured: bool, learn_more: &str) -> Markup {
    html! {
        article class={ "project reveal " (p.theme.class()) @if featured { " featured" } } {
            div.copy {
                p.eyebrow { (p.name) }
                h3 { (p.tagline) }
                p.desc { (p.description) }
                @if !p.tags.is_empty() {
                    ul.tags { @for t in &p.tags { li { (t) } } }
                }
                @if let Some(url) = &p.url {
                    a.more href=(url) rel="noopener" { (learn_more) }
                }
            }
        }
    }
}

pub fn not_found(c: &Content, lang: Lang) -> String {
    let ui = &c.ui;
    let head = html! { meta name="robots" content="noindex"; };
    let body = html! {
        main.notfound {
            p.eyebrow { "404" }
            h1 { (ui.not_found_title) }
            p.intro { (ui.not_found_lead) }
            div.cta { a.btn href=(lang.path()) { (ui.not_found_back) } }
        }
    };
    layout(lang, &format!("404 — {}", c.name), &c.intro, head, body).into_string()
}
