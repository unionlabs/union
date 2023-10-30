#!/usr/bin/env node

const fs = require('fs');
const cheerio = require('cheerio');
const hljs = require('./build/highlight.js');
const Entities = require('html-entities').AllHtmlEntities;
const entities = new Entities();

const githublink = `\
<li class="part-title">\
<a href="https://github.com/dtolnay/cxx">\
<i class="fa fa-github"></i>\
https://github.com/dtolnay/cxx\
</a>\
</li>`;

const opengraph = `\
<meta property="og:image" content="https://cxx.rs/cxx.png" />\
<meta property="og:site_name" content="CXX" />\
<meta property="og:title" content="CXX — safe interop between Rust and C++" />\
<meta name="twitter:image:src" content="https://cxx.rs/cxx.png" />\
<meta name="twitter:site" content="@davidtolnay" />\
<meta name="twitter:card" content="summary" />\
<meta name="twitter:title" content="CXX — safe interop between Rust and C++" />`;

const htmljs = `\
var html = document.querySelector('html');
html.classList.remove('no-js');
html.classList.add('js');`;

const dirs = ['build'];
while (dirs.length) {
  const dir = dirs.pop();
  fs.readdirSync(dir).forEach((entry) => {
    path = dir + '/' + entry;
    const stat = fs.statSync(path);
    if (stat.isDirectory()) {
      dirs.push(path);
      return;
    }

    if (!path.endsWith('.html')) {
      return;
    }

    const index = fs.readFileSync(path, 'utf8');
    const $ = cheerio.load(index, { decodeEntities: false });

    $('head').append(opengraph);
    $('script:nth-of-type(3)').text(htmljs);
    $('nav#sidebar ol.chapter').append(githublink);
    $('head link[href="tomorrow-night.css"]').attr('disabled', true);
    $('head link[href="ayu-highlight.css"]').attr('disabled', true);
    $('button#theme-toggle').attr('style', 'display:none');
    $('pre code').each(function () {
      const node = $(this);
      const langClass = node.attr('class').split(' ', 2)[0];
      if (!langClass.startsWith('language-')) {
        return;
      }
      const lang = langClass.replace('language-', '');
      const lines = node.html().split('\n');
      const boring = lines.map((line) =>
        line.includes('<span class="boring">')
      );
      const ellipsis = lines.map((line) => line.includes('// ...'));
      const target = entities.decode(node.text());
      const highlighted = hljs.highlight(lang, target).value;
      const result = highlighted
        .split('\n')
        .map(function (line, i) {
          if (boring[i]) {
            line = '<span class="boring">' + line;
          } else if (ellipsis[i]) {
            line = '<span class="ellipsis">' + line;
          }
          if (i > 0 && (boring[i - 1] || ellipsis[i - 1])) {
            line = '</span>' + line;
          }
          return line;
        })
        .join('\n');
      node.text(result);
      node.removeClass(langClass);
      if (!node.hasClass('focuscomment')) {
        node.addClass('hidelines');
        node.addClass('hide-boring');
      }
    });
    $('code').each(function () {
      $(this).addClass('hljs');
    });

    const out = $.html();
    fs.writeFileSync(path, out);
  });
}

fs.copyFileSync('build/highlight.css', 'build/tomorrow-night.css');
fs.copyFileSync('build/highlight.css', 'build/ayu-highlight.css');

var bookjs = fs.readFileSync('build/book.js', 'utf8');
bookjs = bookjs.replace('set_theme(theme, false);', '');
fs.writeFileSync('build/book.js', bookjs);
