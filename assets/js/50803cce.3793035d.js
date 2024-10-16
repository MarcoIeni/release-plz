"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[7706],{5785:(e,n,i)=>{i.r(n),i.d(n,{assets:()=>l,contentTitle:()=>a,default:()=>h,frontMatter:()=>t,metadata:()=>d,toc:()=>o});var r=i(4848),s=i(8453);const t={},a="Examples",d={id:"changelog/examples",title:"Examples",description:"Release-plz comes with a default changelog configuration that adheres to the",source:"@site/docs/changelog/examples.md",sourceDirName:"changelog",slug:"/changelog/examples",permalink:"/docs/changelog/examples",draft:!1,unlisted:!1,editUrl:"https://github.com/MarcoIeni/release-plz/tree/main/website/docs/changelog/examples.md",tags:[],version:"current",frontMatter:{},sidebar:"tutorialSidebar",previous:{title:"Format",permalink:"/docs/changelog/format"},next:{title:"Tips And Tricks",permalink:"/docs/changelog/tips-and-tricks"}},l={},o=[{value:"Release-plz default",id:"release-plz-default",level:2},{value:"[Unreleased]",id:"unreleased",level:2},{value:"1.0.1 - 2021-07-18",id:"101---2021-07-18",level:2},{value:"Added",id:"added",level:3},{value:"Changed",id:"changed",level:3},{value:"[1.0.0] - 2021-07-18",id:"100---2021-07-18",level:2},{value:"Added",id:"added-1",level:3},{value:"Fixed",id:"fixed",level:3},{value:"Styled and scoped",id:"styled-and-scoped",level:2},{value:"[Unreleased]",id:"unreleased-1",level:2},{value:"1.0.1",id:"101",level:2},{value:"\ud83d\ude9c Refactor",id:"-refactor",level:3},{value:"\u2699\ufe0f Miscellaneous Tasks",id:"\ufe0f-miscellaneous-tasks",level:3},{value:"[1.0.0] - 2021-07-18",id:"100---2021-07-18-1",level:2},{value:"\u26f0\ufe0f  Features",id:"\ufe0f--features",level:3},{value:"\ud83d\udc1b Bug Fixes",id:"-bug-fixes",level:3},{value:"\ud83d\udcda Documentation",id:"-documentation",level:3},{value:"Release-plz default + contributors",id:"release-plz-default--contributors",level:2},{value:"[Unreleased]",id:"unreleased-2",level:2},{value:"1.0.1 - 2021-07-18",id:"101---2021-07-18-1",level:2},{value:"Added",id:"added-2",level:3},{value:"Changed",id:"changed-1",level:3},{value:"Contributors",id:"contributors",level:3},{value:"[1.0.0] - 2021-07-18",id:"100---2021-07-18-2",level:2},{value:"Added",id:"added-3",level:3},{value:"Fixed",id:"fixed-1",level:3},{value:"Contributors",id:"contributors-1",level:3}];function c(e){const n={a:"a",admonition:"admonition",code:"code",h1:"h1",h2:"h2",h3:"h3",header:"header",li:"li",p:"p",pre:"pre",ul:"ul",...(0,s.R)(),...e.components},{Details:i}=n;return i||function(e,n){throw new Error("Expected "+(n?"component":"object")+" `"+e+"` to be defined: you likely forgot to import, pass, or provide it.")}("Details",!0),(0,r.jsxs)(r.Fragment,{children:[(0,r.jsx)(n.header,{children:(0,r.jsx)(n.h1,{id:"examples",children:"Examples"})}),"\n",(0,r.jsxs)(n.p,{children:["Release-plz comes with a default changelog configuration that adheres to the\n",(0,r.jsx)(n.a,{href:"https://keepachangelog.com/en/1.0.0/",children:"Keep a Changelog"})," specification.\nYou can customize the changelog format by editing the\n",(0,r.jsx)(n.a,{href:"/docs/config#the-changelog-section",children:(0,r.jsx)(n.code,{children:"[changelog]"})})," section of the configuration file."]}),"\n",(0,r.jsx)(n.p,{children:"In the following there are some examples of changelog configurations that you can\nuse to take inspiration from. \u2728"}),"\n",(0,r.jsxs)(n.p,{children:["If you want to contribute your cool template,\n",(0,r.jsx)(n.a,{href:"https://github.com/MarcoIeni/release-plz/blob/main/CONTRIBUTING.md",children:"open a PR"}),"! \ud83d\ude4f"]}),"\n",(0,r.jsxs)(n.admonition,{type:"info",children:[(0,r.jsxs)(n.p,{children:["All examples based on the following ",(0,r.jsx)(n.a,{href:"https://github.com/orhun/git-cliff-readme-example",children:"Git\nhistory"}),":"]}),(0,r.jsx)(n.pre,{children:(0,r.jsx)(n.code,{className:"language-text",children:"* df6aef4 (HEAD -> master) feat(cache): use cache while fetching pages\n* a9d4050 feat(config): support multiple file formats\n* 06412ac (tag: v1.0.1) chore(release): add release script\n* e4fd3cf refactor(parser): expose string functions\n* ad27b43 (tag: v1.0.0) docs(example)!: add tested usage example\n* 9add0d4 fix(args): rename help argument due to conflict\n* a140cef feat(parser): add ability to parse arrays\n* 81fbc63 docs(project): add README.md\n* a78bc36 Initial commit\n"})})]}),"\n",(0,r.jsx)(n.h2,{id:"release-plz-default",children:"Release-plz default"}),"\n",(0,r.jsx)(n.p,{children:"Release-plz default configuration, purely here as a reference."}),"\n",(0,r.jsxs)(i,{children:[(0,r.jsx)("summary",{children:"TOML configuration"}),(0,r.jsx)(n.pre,{children:(0,r.jsx)(n.code,{className:"language-toml",children:'[changelog]\nheader = """# Changelog\n\nAll notable changes to this project will be documented in this file.\n\nThe format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),\nand this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n## [Unreleased]\n"""\n\nbody = """\n\n## [{{ version | trim_start_matches(pat="v") }}]\\\n    {%- if release_link -%}\\\n        ({{ release_link }})\\\n    {% endif %} \\\n    - {{ timestamp | date(format="%Y-%m-%d") }}\n{% for group, commits in commits | group_by(attribute="group") %}\n### {{ group | upper_first }}\n    {% for commit in commits %}\n        {%- if commit.scope -%}\n            - *({{commit.scope}})* {% if commit.breaking %}[**breaking**] {% endif %}\\\n                {{ commit.message }}\\\n                {%- if commit.links %} \\\n                    ({% for link in commit.links %}[{{link.text}}]({{link.href}}) {% endfor -%})\\\n                {% endif %}\n        {% else -%}\n            - {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }}\n        {% endif -%}\n    {% endfor -%}\n{% endfor %}\n"""\n\ncommit_parsers = [\n  { message = "^feat", group = "added" },\n  { message = "^changed", group = "changed" },\n  { message = "^deprecated", group = "deprecated" },\n  { message = "^fix", group = "fixed" },\n  { message = "^security", group = "security" },\n  { message = "^.*", group = "other" },\n]\n'})})]}),"\n",(0,r.jsxs)(i,{children:[(0,r.jsx)("summary",{children:"Raw Output"}),(0,r.jsx)(n.pre,{children:(0,r.jsx)(n.code,{className:"language-md",children:"# Changelog\n\nAll notable changes to this project will be documented in this file.\n\nThe format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),\nand this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n## [Unreleased]\n\n## [1.0.1](https://github.com/orhun/git-cliff-readme-example/compare/v1.0.0...v1.0.1) - 2021-07-18\n\n### Added\n\n- Add release script\n\n### Changed\n\n- Expose string functions\n\n## [1.0.0] - 2021-07-18\n\n### Added\n\n- Add README.md\n- Add ability to parse arrays\n- Add tested usage example\n\n### Fixed\n\n- Rename help argument due to conflict\n"})})]}),"\n",(0,r.jsxs)(i,{children:[(0,r.jsx)("summary",{children:"Rendered Output"}),(0,r.jsx)(n.h1,{id:"changelog",children:"Changelog"}),(0,r.jsxs)(n.p,{children:["All notable changes to this project will be documented in this file.\nThe format is based on ",(0,r.jsx)(n.a,{href:"https://keepachangelog.com/en/1.0.0/",children:"Keep a Changelog"}),",\nand this project adheres to ",(0,r.jsx)(n.a,{href:"https://semver.org/spec/v2.0.0.html",children:"Semantic Versioning"}),"."]}),(0,r.jsx)(n.h2,{id:"unreleased",children:"[Unreleased]"}),(0,r.jsxs)(n.h2,{id:"101---2021-07-18",children:[(0,r.jsx)(n.a,{href:"https://github.com/orhun/git-cliff-readme-example/compare/v1.0.0...v1.0.1",children:"1.0.1"})," - 2021-07-18"]}),(0,r.jsx)(n.h3,{id:"added",children:"Added"}),(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsx)(n.li,{children:"Add release script"}),"\n"]}),(0,r.jsx)(n.h3,{id:"changed",children:"Changed"}),(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsx)(n.li,{children:"Expose string functions"}),"\n"]}),(0,r.jsx)(n.h2,{id:"100---2021-07-18",children:"[1.0.0] - 2021-07-18"}),(0,r.jsx)(n.h3,{id:"added-1",children:"Added"}),(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsx)(n.li,{children:"Add README.md"}),"\n",(0,r.jsx)(n.li,{children:"Add ability to parse arrays"}),"\n",(0,r.jsx)(n.li,{children:"Add tested usage example"}),"\n"]}),(0,r.jsx)(n.h3,{id:"fixed",children:"Fixed"}),(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsx)(n.li,{children:"Rename help argument due to conflict"}),"\n"]})]}),"\n",(0,r.jsx)(n.h2,{id:"styled-and-scoped",children:"Styled and scoped"}),"\n",(0,r.jsxs)(i,{children:[(0,r.jsx)("summary",{children:"TOML configuration"}),(0,r.jsx)(n.pre,{children:(0,r.jsx)(n.code,{className:"language-toml",children:'[changelog]\nheader = """# Changelog\n\n## [Unreleased]\n"""\n\nbody = """\n\n{% macro print_commit(commit) -%}\n    - {% if commit.scope %}*({{ commit.scope }})* {% endif %}\\\n      {% if commit.breaking %}[**breaking**] {% endif %}\\\n      {{ commit.message | upper_first }} - \\\n      ([{{ commit.id | truncate(length=7, end="") }}]({{ remote.link }}/commit/{{ commit.id }}))\\\n{% endmacro -%}\n\n{% if version %}\\\n    {% if previous.version %}\\\n        ## [{{ version | trim_start_matches(pat="v") }}]({{ release_link }})\n    {% else %}\\\n        ## [{{ version | trim_start_matches(pat="v") }}]\n    {% endif %}\\\n{% endif %}\\\n\n{% for group, commits in commits\n| filter(attribute="merge_commit", value=false)\n| unique(attribute="message")\n| group_by(attribute="group") %}\n    ### {{ group | striptags | trim | upper_first }}\n    {% for commit in commits\n    | filter(attribute="scope")\n    | sort(attribute="scope") %}\n        {{ self::print_commit(commit=commit) }}\n    {%- endfor -%}\n    {% raw %}\\n{% endraw %}\\\n    {%- for commit in commits %}\n        {%- if not commit.scope -%}\n            {{ self::print_commit(commit=commit) }}\n        {% endif -%}\n    {% endfor -%}\n{% endfor %}\\n\n"""\n\ncommit_parsers = [\n  { message = "^feat", group = "\x3c!-- 0 --\x3e\u26f0\ufe0f Features" },\n  { message = "^fix", group = "\x3c!-- 1 --\x3e\ud83d\udc1b Bug Fixes" },\n  { message = "^doc", group = "\x3c!-- 3 --\x3e\ud83d\udcda Documentation" },\n  { message = "^perf", group = "\x3c!-- 4 --\x3e\u26a1 Performance" },\n  { message = "^refactor\\\\(clippy\\\\)", skip = true },\n  { message = "^refactor", group = "\x3c!-- 2 --\x3e\ud83d\ude9c Refactor" },\n  { message = "^style", group = "\x3c!-- 5 --\x3e\ud83c\udfa8 Styling" },\n  { message = "^test", group = "\x3c!-- 6 --\x3e\ud83e\uddea Testing" },\n  { message = "^chore\\\\(release\\\\):", skip = true },\n  { message = "^chore: release", skip = true },\n  { message = "^chore\\\\(deps.*\\\\)", skip = true },\n  { message = "^chore\\\\(pr\\\\)", skip = true },\n  { message = "^chore\\\\(pull\\\\)", skip = true },\n  { message = "^chore\\\\(npm\\\\).*yarn\\\\.lock", skip = true },\n  { message = "^chore|^ci", group = "\x3c!-- 7 --\x3e\u2699\ufe0f Miscellaneous Tasks" },\n  { body = ".*security", group = "\x3c!-- 8 --\x3e\ud83d\udee1\ufe0f Security" },\n  { message = "^revert", group = "\x3c!-- 9 --\x3e\u25c0\ufe0f Revert" },\n]\n\nlink_parsers = [\n  { pattern = "#(\\\\d+)", href = "{{ remote.link }}/issues/$1" },\n  { pattern = "RFC(\\\\d+)", text = "ietf-rfc$1", href = "https://datatracker.ietf.org/doc/html/rfc$1" },\n]\n'})})]}),"\n",(0,r.jsxs)(i,{children:[(0,r.jsx)("summary",{children:"Raw Output"}),(0,r.jsx)(n.pre,{children:(0,r.jsx)(n.code,{className:"language-md",children:"# Changelog\n\n## [Unreleased]\n\n## [1.0.1](https://github.com/orhun/git-cliff-readme-example/compare/v1.0.0...v1.0.1)\n\n### \ud83d\ude9c Refactor\n\n- refactor(parser): expose string functions ([e4fd3cf](e4fd3cf8e2e6f49c0b57f66416e886c37cbb3715))\n\n### \u2699\ufe0f Miscellaneous Tasks\n\n- chore(release): add release script ([06412ac](06412ac1dd4071006c465dde6597a21d4367a158))\n\n## [1.0.0] - 2021-07-18\n\n### \u26f0\ufe0f  Features\n\n- feat(parser): add ability to parse arrays ([a140cef](a140cef0405e0bcbfb5de44ff59e091527d91b38))\n\n### \ud83d\udc1b Bug Fixes\n\n- fix(args): rename help argument due to conflict ([9add0d4](9add0d4616dc95a6ea8b01d5e4d233876b6e5e00))\n\n### \ud83d\udcda Documentation\n\n- docs(project): add README.md ([81fbc63](81fbc6365484abf0b4f4b05d384175763ad8db44))\n- docs(example)!: add tested usage example ([ad27b43](ad27b43e8032671afb4809a1a3ecf12f45c60e0e))\n"})})]}),"\n",(0,r.jsxs)(i,{children:[(0,r.jsx)("summary",{children:"Rendered Output"}),(0,r.jsx)(n.h1,{id:"changelog-1",children:"Changelog"}),(0,r.jsxs)(n.p,{children:["All notable changes to this project will be documented in this file.\nThe format is based on ",(0,r.jsx)(n.a,{href:"https://keepachangelog.com/en/1.0.0/",children:"Keep a Changelog"}),",\nand this project adheres to ",(0,r.jsx)(n.a,{href:"https://semver.org/spec/v2.0.0.html",children:"Semantic Versioning"}),"."]}),(0,r.jsx)(n.h2,{id:"unreleased-1",children:"[Unreleased]"}),(0,r.jsx)(n.h2,{id:"101",children:(0,r.jsx)(n.a,{href:"https://github.com/orhun/git-cliff-readme-example/compare/v1.0.0...v1.0.1",children:"1.0.1"})}),(0,r.jsx)(n.h3,{id:"-refactor",children:"\ud83d\ude9c Refactor"}),(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsx)(n.li,{children:"refactor(parser): expose string functions (e4fd3cf)"}),"\n"]}),(0,r.jsx)(n.h3,{id:"\ufe0f-miscellaneous-tasks",children:"\u2699\ufe0f Miscellaneous Tasks"}),(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsx)(n.li,{children:"chore(release): add release script (06412ac)"}),"\n"]}),(0,r.jsx)(n.h2,{id:"100---2021-07-18-1",children:"[1.0.0] - 2021-07-18"}),(0,r.jsx)(n.h3,{id:"\ufe0f--features",children:"\u26f0\ufe0f  Features"}),(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsx)(n.li,{children:"feat(parser): add ability to parse arrays (a140cef)"}),"\n"]}),(0,r.jsx)(n.h3,{id:"-bug-fixes",children:"\ud83d\udc1b Bug Fixes"}),(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsx)(n.li,{children:"fix(args): rename help argument due to conflict (9add0d4)"}),"\n"]}),(0,r.jsx)(n.h3,{id:"-documentation",children:"\ud83d\udcda Documentation"}),(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsx)(n.li,{children:"docs(project): add README.md (81fbc63)"}),"\n",(0,r.jsx)(n.li,{children:"docs(example)!: add tested usage example (ad27b43)"}),"\n"]})]}),"\n",(0,r.jsx)(n.h2,{id:"release-plz-default--contributors",children:"Release-plz default + contributors"}),"\n",(0,r.jsx)(n.p,{children:"Like Release-plz default configuration, but it also shows the\nGitHub/Gitea/GitLab username of the contributors."}),"\n",(0,r.jsxs)(i,{children:[(0,r.jsx)("summary",{children:"TOML configuration"}),(0,r.jsx)(n.pre,{children:(0,r.jsx)(n.code,{className:"language-toml",children:'[changelog]\nbody = """\n\n## [{{ version | trim_start_matches(pat="v") }}]\\\n    {%- if release_link -%}\\\n        ({{ release_link }})\\\n    {% endif %} \\\n    - {{ timestamp | date(format="%Y-%m-%d") }}\n{% for group, commits in commits | group_by(attribute="group") %}\n### {{ group | upper_first }}\n    {% for commit in commits %}\n        {%- if commit.scope -%}\n            - *({{commit.scope}})* {% if commit.breaking %}[**breaking**] {% endif %}\\\n# highlight-next-line\n                {{ commit.message }}{{ self::username(commit=commit) }}\\\n                {%- if commit.links %} \\\n                    ({% for link in commit.links %}[{{link.text}}]({{link.href}}) {% endfor -%})\\\n                {% endif %}\n        {% else -%}\n# highlight-next-line\n            - {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }}{{ self::username(commit=commit) }}{{ self::pr(commit=commit) }}\n        {% endif -%}\n    {% endfor -%}\n{% endfor %}\n# highlight-start\n{%- if remote.contributors %}\n### Contributors\n{% for contributor in remote.contributors %}\n    * @{{ contributor.username }}\n{%- endfor %}\n{% endif -%}\n{%- macro username(commit) -%}\n    {% if commit.remote.username %} (by @{{ commit.remote.username }}){% endif -%}\n{% endmacro -%}\n{%- macro pr(commit) -%}\n    {% if commit.remote.pr_number %} - #{{ commit.remote.pr_number }}{% endif -%}\n{% endmacro -%}\n# highlight-end\n"""\n'})})]}),"\n",(0,r.jsxs)(i,{children:[(0,r.jsx)("summary",{children:"Raw Output"}),(0,r.jsx)(n.pre,{children:(0,r.jsx)(n.code,{className:"language-md",children:"# Changelog\n\nAll notable changes to this project will be documented in this file.\n\nThe format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),\nand this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n## [Unreleased]\n\n## [1.0.1](https://github.com/orhun/git-cliff-readme-example/compare/v1.0.0...v1.0.1) - 2021-07-18\n\n### Added\n\n- Add release script (by @orhun)\n\n### Changed\n\n- Expose string functions (by @orhun)\n\n### Contributors\n\n* @orhun\n\n## [1.0.0] - 2021-07-18\n\n### Added\n\n- Add README.md (by @orhun)\n- Add ability to parse arrays (by @orhun)\n- Add tested usage example (by @orhun)\n\n### Fixed\n\n- Rename help argument due to conflict (by @orhun)\n\n### Contributors\n\n* @orhun\n"})})]}),"\n",(0,r.jsxs)(i,{children:[(0,r.jsx)("summary",{children:"Rendered Output"}),(0,r.jsx)(n.h1,{id:"changelog-2",children:"Changelog"}),(0,r.jsxs)(n.p,{children:["All notable changes to this project will be documented in this file.\nThe format is based on ",(0,r.jsx)(n.a,{href:"https://keepachangelog.com/en/1.0.0/",children:"Keep a Changelog"}),",\nand this project adheres to ",(0,r.jsx)(n.a,{href:"https://semver.org/spec/v2.0.0.html",children:"Semantic Versioning"}),"."]}),(0,r.jsx)(n.h2,{id:"unreleased-2",children:"[Unreleased]"}),(0,r.jsxs)(n.h2,{id:"101---2021-07-18-1",children:[(0,r.jsx)(n.a,{href:"https://github.com/orhun/git-cliff-readme-example/compare/v1.0.0...v1.0.1",children:"1.0.1"})," - 2021-07-18"]}),(0,r.jsx)(n.h3,{id:"added-2",children:"Added"}),(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsx)(n.li,{children:"Add release script (by @orhun)"}),"\n"]}),(0,r.jsx)(n.h3,{id:"changed-1",children:"Changed"}),(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsx)(n.li,{children:"Expose string functions (by @orhun)"}),"\n"]}),(0,r.jsx)(n.h3,{id:"contributors",children:"Contributors"}),(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsx)(n.li,{children:"@orhun"}),"\n"]}),(0,r.jsx)(n.h2,{id:"100---2021-07-18-2",children:"[1.0.0] - 2021-07-18"}),(0,r.jsx)(n.h3,{id:"added-3",children:"Added"}),(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsx)(n.li,{children:"Add README.md (by @orhun)"}),"\n",(0,r.jsx)(n.li,{children:"Add ability to parse arrays (by @orhun)"}),"\n",(0,r.jsx)(n.li,{children:"Add tested usage example (by @orhun)"}),"\n"]}),(0,r.jsx)(n.h3,{id:"fixed-1",children:"Fixed"}),(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsx)(n.li,{children:"Rename help argument due to conflict (by @orhun)"}),"\n"]}),(0,r.jsx)(n.h3,{id:"contributors-1",children:"Contributors"}),(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsx)(n.li,{children:"@orhun"}),"\n"]})]})]})}function h(e={}){const{wrapper:n}={...(0,s.R)(),...e.components};return n?(0,r.jsx)(n,{...e,children:(0,r.jsx)(c,{...e})}):c(e)}},8453:(e,n,i)=>{i.d(n,{R:()=>a,x:()=>d});var r=i(6540);const s={},t=r.createContext(s);function a(e){const n=r.useContext(t);return r.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function d(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:a(e.components),r.createElement(t.Provider,{value:n},e.children)}}}]);