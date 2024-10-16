"use strict";(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[8880],{6508:(e,n,s)=>{s.r(n),s.d(n,{assets:()=>o,contentTitle:()=>a,default:()=>h,frontMatter:()=>t,metadata:()=>l,toc:()=>c});var r=s(4848),i=s(8453);const t={},a="release",l={id:"usage/release",title:"release",description:"The release-plz release command releases all the unpublished packages.",source:"@site/docs/usage/release.md",sourceDirName:"usage",slug:"/usage/release",permalink:"/docs/usage/release",draft:!1,unlisted:!1,editUrl:"https://github.com/MarcoIeni/release-plz/tree/main/website/docs/usage/release.md",tags:[],version:"current",frontMatter:{},sidebar:"tutorialSidebar",previous:{title:"release-pr",permalink:"/docs/usage/release-pr"},next:{title:"init",permalink:"/docs/usage/init"}},o={},c=[{value:"Git Backends",id:"git-backends",level:2},{value:"Gitlab",id:"gitlab",level:3},{value:"Gitea",id:"gitea",level:3},{value:"Json output",id:"json-output",level:2},{value:"The <code>tag</code> field",id:"the-tag-field",level:3},{value:"The <code>prs</code> field",id:"the-prs-field",level:3},{value:"What commit is released",id:"what-commit-is-released",level:2}];function d(e){const n={a:"a",admonition:"admonition",blockquote:"blockquote",code:"code",h1:"h1",h2:"h2",h3:"h3",header:"header",li:"li",ol:"ol",p:"p",pre:"pre",ul:"ul",...(0,i.R)(),...e.components},{Details:s}=n;return s||function(e,n){throw new Error("Expected "+(n?"component":"object")+" `"+e+"` to be defined: you likely forgot to import, pass, or provide it.")}("Details",!0),(0,r.jsxs)(r.Fragment,{children:[(0,r.jsx)(n.header,{children:(0,r.jsx)(n.h1,{id:"release",children:"release"})}),"\n",(0,r.jsxs)(n.p,{children:["The ",(0,r.jsx)(n.code,{children:"release-plz release"})," command releases all the unpublished packages."]}),"\n",(0,r.jsxs)(n.blockquote,{children:["\n",(0,r.jsxs)(n.p,{children:["For example, let's say you have a workspace with two packages: ",(0,r.jsx)(n.code,{children:"pkg-a"}),"\n(version 0.3.1) and ",(0,r.jsx)(n.code,{children:"pkg-b"})," (version 0.2.2).\nThe crates.io registry contains ",(0,r.jsx)(n.code,{children:"pkg-a"})," version 0.3.1, but it doesn't contain\n",(0,r.jsx)(n.code,{children:"pkg-b"})," version 0.2.2 because you didn't publish this version yet.\nIn this case, release-plz would release ",(0,r.jsx)(n.code,{children:"pkg-b"}),"."]}),"\n"]}),"\n",(0,r.jsx)(n.p,{children:"For every release, release-plz:"}),"\n",(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsxs)(n.li,{children:["Creates a git tag named ",(0,r.jsx)(n.code,{children:"<package_name>-v<version>"})," (e.g. ",(0,r.jsx)(n.code,{children:"tokio-v1.8.1"}),").\n",(0,r.jsx)(n.code,{children:"<package_name>-"})," is omitted if there's only one package to publish."]}),"\n",(0,r.jsxs)(n.li,{children:["Publishes the package to the cargo registry by running ",(0,r.jsx)(n.code,{children:"cargo publish"}),"."]}),"\n",(0,r.jsx)(n.li,{children:"Publishes a GitHub/Gitea/GitLab release based on the git tag."}),"\n"]}),"\n",(0,r.jsx)(n.admonition,{type:"info",children:(0,r.jsxs)(n.p,{children:[(0,r.jsx)(n.code,{children:"release-plz release"})," doesn't edit your ",(0,r.jsx)(n.code,{children:"Cargo.toml"})," files and doesn't\npush new commits. It releases the packages as they are in your repository.\nFor this reason, you typically use the ",(0,r.jsx)(n.code,{children:"release-plz release"})," command in the main branch\nafter you run ",(0,r.jsx)(n.code,{children:"release-plz update"}),"\nor you merge a pull request opened with ",(0,r.jsx)(n.code,{children:"release-plz release-pr"}),"."]})}),"\n",(0,r.jsxs)(n.p,{children:["If all packages are already published, the ",(0,r.jsx)(n.code,{children:"release-plz release"})," command does nothing."]}),"\n",(0,r.jsxs)(n.p,{children:["To learn more, run ",(0,r.jsx)(n.code,{children:"release-plz release --help"}),"."]}),"\n",(0,r.jsx)(n.h2,{id:"git-backends",children:"Git Backends"}),"\n",(0,r.jsxs)(n.p,{children:["GitHub is the default release-plz backend. You can use the ",(0,r.jsx)(n.code,{children:"--backend"})," flag to\nspecify a different backend."]}),"\n",(0,r.jsx)(n.h3,{id:"gitlab",children:"Gitlab"}),"\n",(0,r.jsxs)(n.p,{children:[(0,r.jsx)(n.code,{children:"release-plz release"})," also supports creating releases for repositories hosted on Gitlab with\nthe ",(0,r.jsx)(n.code,{children:"--backend gitlab"})," option:"]}),"\n",(0,r.jsx)(n.p,{children:"You need to create a token in your Gitlab repo (Settings/Access Tokens) with the following\npermissions:"}),"\n",(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsxs)(n.li,{children:["Role: ",(0,r.jsx)(n.code,{children:"Maintainer"})," or higher"]}),"\n",(0,r.jsxs)(n.li,{children:["Scopes:","\n",(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsxs)(n.li,{children:[(0,r.jsx)(n.code,{children:"api"})," (to create a release)"]}),"\n",(0,r.jsxs)(n.li,{children:[(0,r.jsx)(n.code,{children:"write_repository"})," (to create tag)"]}),"\n"]}),"\n"]}),"\n"]}),"\n",(0,r.jsxs)(n.p,{children:["See the Gitlab ",(0,r.jsx)(n.a,{href:"https://docs.gitlab.com/ee/user/project/settings/project_access_tokens.html",children:"project access tokens"}),"\ndocs."]}),"\n",(0,r.jsxs)(n.p,{children:["Then you can run ",(0,r.jsx)(n.code,{children:"release-plz release"})," with the following arguments:"]}),"\n",(0,r.jsx)(n.p,{children:(0,r.jsx)(n.code,{children:"release-plz release --backend gitlab --git-token <gitlab_token>"})}),"\n",(0,r.jsx)(n.h3,{id:"gitea",children:"Gitea"}),"\n",(0,r.jsxs)(n.p,{children:[(0,r.jsx)(n.code,{children:"releases-plz"})," supports creating releases on Gitea with the ",(0,r.jsx)(n.code,{children:"--backend gitea"})," option."]}),"\n",(0,r.jsx)(n.p,{children:"TODO: document how to create a token on Gitea."}),"\n",(0,r.jsxs)(n.p,{children:["Then you can run ",(0,r.jsx)(n.code,{children:"release-plz release"})," in Gitea CI with the following arguments:"]}),"\n",(0,r.jsx)(n.p,{children:(0,r.jsx)(n.code,{children:"release-plz release --backend gitea --git-token <gitea_token>"})}),"\n",(0,r.jsx)(n.h2,{id:"json-output",children:"Json output"}),"\n",(0,r.jsxs)(n.p,{children:["You can get info about the outcome of this command by appending ",(0,r.jsx)(n.code,{children:"-o json"})," to the command.\nStdout will contain info about the release:"]}),"\n",(0,r.jsx)(n.pre,{children:(0,r.jsx)(n.code,{className:"language-json",children:'{\n  "releases": [\n    {\n      "package_name": "<package_name>",\n      "prs": "<prs>",\n      "tag": "<tag_name>",\n      "version": "<version>"\n    }\n  ]\n}\n'})}),"\n",(0,r.jsx)(n.p,{children:"Example:"}),"\n",(0,r.jsx)(n.pre,{children:(0,r.jsx)(n.code,{className:"language-json",children:'{\n  "releases": [\n    {\n      "package_name": "my_crate",\n      "prs": [\n        {\n          "html_url": "https://github.com/user/proj/pull/1439",\n          "number": 1439\n        }\n      ],\n      "tag": "v0.1.0",\n      "version": "0.1.0"\n    }\n  ]\n}\n'})}),"\n",(0,r.jsxs)(n.p,{children:["If release-plz didn't release any packages, the ",(0,r.jsx)(n.code,{children:"releases"})," array will be empty."]}),"\n",(0,r.jsxs)(n.h3,{id:"the-tag-field",children:["The ",(0,r.jsx)(n.code,{children:"tag"})," field"]}),"\n",(0,r.jsxs)(n.p,{children:["The ",(0,r.jsx)(n.code,{children:"tag"})," field is present even if the user disabled the tag creation with the\n",(0,r.jsx)(n.a,{href:"/docs/config#the-git_tag_enable-field",children:(0,r.jsx)(n.code,{children:"git_tag_enable"})})," field.\nThis is because the user might want to use the tag name to create the tag\nby themselves."]}),"\n",(0,r.jsxs)(n.h3,{id:"the-prs-field",children:["The ",(0,r.jsx)(n.code,{children:"prs"})," field"]}),"\n",(0,r.jsxs)(n.p,{children:[(0,r.jsx)(n.code,{children:"prs"})," is an array of PRs present in the changelog body of the release.\nUsually, they are the PRs containing the changes that were released."]}),"\n",(0,r.jsx)(n.p,{children:"Each entry of the array is an object containing:"}),"\n",(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsxs)(n.li,{children:[(0,r.jsx)(n.code,{children:"html_url"}),": The URL of the PR."]}),"\n",(0,r.jsxs)(n.li,{children:[(0,r.jsx)(n.code,{children:"number"}),": The number of the PR."]}),"\n"]}),"\n",(0,r.jsx)(n.h2,{id:"what-commit-is-released",children:"What commit is released"}),"\n",(0,r.jsx)(n.admonition,{type:"info",children:(0,r.jsx)(n.p,{children:"This is an advanced section that describes certain design choices of release-plz,\nmainly relevant for repositories with a merge queue enabled.\nYou can skip it if you are just getting started with release-plz\nor if you are the only maintainer of your repository."})}),"\n",(0,r.jsxs)(n.p,{children:["To avoid race conditions when the release PR is merged,\n",(0,r.jsx)(n.code,{children:"release-plz release"})," does a ",(0,r.jsx)(n.code,{children:"git checkout"})," to the latest commit of the PR\nbefore releasing (if the commit of the PR exists in the main branch)."]}),"\n",(0,r.jsx)(n.p,{children:"Depending on the merge strategy you use, this can have different effects:"}),"\n",(0,r.jsxs)(n.ul,{children:["\n",(0,r.jsxs)(n.li,{children:["If you merge with the\n",(0,r.jsx)(n.a,{href:"https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/about-merge-methods-on-github#squashing-your-merge-commits",children:"squash and merge"}),"\nstrategy, the ",(0,r.jsx)(n.code,{children:"git checkout"})," won't happen because when you merge the PR to the main branch,\nGitHub creates a new commit,\nso release-plz won't find the commit of the PR and will release the latest commit\nof the main branch."]}),"\n",(0,r.jsxs)(n.li,{children:["If you merge with the\n",(0,r.jsx)(n.a,{href:"https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/about-merge-methods-on-github",children:"merge"}),'\nstrategy (the GitHub default), release-plz will release the last commit\nof the PR instead of the "Merge pull request" commit created by GitHub.']}),"\n"]}),"\n",(0,r.jsx)(n.p,{children:"Takeaway: if you are concerned about PRs being released by mistake\n(because you have a merge queue enabled or because your repository\nhas multiple maintainers), you should merge release PRs with the\ndefault merge strategy.\nRelease-plz will handle the rest, avoiding race conditions happening when\nthe release PR is merged immediately after other PRs that aren't meant to be released. \ud83d\udc4d"}),"\n",(0,r.jsxs)(n.p,{children:["Here's an example of race condition that could happen if release-plz\ndidn't do the ",(0,r.jsx)(n.code,{children:"git checkout"})," to the latest PR commit:"]}),"\n",(0,r.jsxs)(s,{children:[(0,r.jsx)("summary",{children:"Merge queue example"}),(0,r.jsxs)(n.ol,{children:["\n",(0,r.jsxs)(n.li,{children:["Person A adds PR 20 to the merge queue (e.g. ",(0,r.jsx)(n.code,{children:"@bors r+"}),")."]}),"\n",(0,r.jsx)(n.li,{children:"Person B adds PR 21 to the merge queue."}),"\n",(0,r.jsxs)(n.li,{children:["PR 20 is merged into ",(0,r.jsx)(n.code,{children:"main"}),"."]}),"\n",(0,r.jsx)(n.li,{children:"Person A sees PR 20 is merged and adds the release PR (PR 22) to the merge queue."}),"\n",(0,r.jsxs)(n.li,{children:["PR 21 is merged into ",(0,r.jsx)(n.code,{children:"main"}),"."]}),"\n",(0,r.jsxs)(n.li,{children:["PR 22 is merged into ",(0,r.jsx)(n.code,{children:"main"}),". The ",(0,r.jsx)(n.code,{children:"release-plz release-pr"})," workflow for PR 21 didn't\nfinish in time, so the release PR is out of date."]}),"\n",(0,r.jsxs)(n.li,{children:[(0,r.jsx)(n.code,{children:"main"}),"'s workflow runs that does the publish for PR 22."]}),"\n"]}),(0,r.jsx)(n.pre,{children:(0,r.jsx)(n.code,{className:"language-mermaid",children:'flowchart LR\n  pr20(["PR 20 (fix)"])\n  pr20_merge["main\n  (PR 20 merge commit)"]\n  pr21(["PR 21 (breaking change)"])\n  pr21_merge["main\n  (PR 21 merge commit)"]\n  pr22(["PR 22 (release)"])\n  pr22_merge["main\n  (PR 22 merge commit)"]\n  main --\x3e pr20\n  main --\x3e pr20_merge\n  pr20 --\x3e pr20_merge\n  main --\x3e pr21\n  pr20_merge --\x3e pr21_merge\n  pr21 --\x3e pr21_merge\n  pr20_merge --\x3e pr22\n  pr22 --\x3e pr22_merge\n  pr21_merge --\x3e pr22_merge\n'})}),(0,r.jsxs)(n.p,{children:["This means that if release-plz didn't do the ",(0,r.jsx)(n.code,{children:"git checkout"}),",\nyour release would include changes from PR 21 which will be missing from the changelog\nand might contain breaking changes."]}),(0,r.jsxs)(n.p,{children:["However, thanks to the ",(0,r.jsx)(n.code,{children:"git checkout"})," to the latest commit of the PR,\nif the release PR was merged into ",(0,r.jsx)(n.code,{children:"main"})," with the default merge strategy, then\nthis race condition doesn't happen because the ancestor of the latest commit\nof PR 22 is PR 20, not PR 21."]})]})]})}function h(e={}){const{wrapper:n}={...(0,i.R)(),...e.components};return n?(0,r.jsx)(n,{...e,children:(0,r.jsx)(d,{...e})}):d(e)}},8453:(e,n,s)=>{s.d(n,{R:()=>a,x:()=>l});var r=s(6540);const i={},t=r.createContext(i);function a(e){const n=r.useContext(t);return r.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function l(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(i):e.components||i:a(e.components),r.createElement(t.Provider,{value:n},e.children)}}}]);