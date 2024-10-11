---
title: "BadgerDB Documentation"
date: 2020-07-06T17:43:29+05:30
draft: false
---

<div class="landing">
  <div class="hero">
    <p>
BadgerDB is an embeddable, persistent, and fast key-value (KV) database written
in pure Go. It is the underlying database for <a href="https://dgraph.io">Dgraph</a>, a
fast, distributed graph database. It's meant to be a performant alternative to
non-Go-based key-value stores like RocksDB.
    </p>
    <img class="hero-deco" src="/images/diggy-shadow.png" />
  </div>
  <div class="item">
    <div class="icon"><i class="lni lni-play" aria-hidden="true"></i></div>
    <a  href="{{< relref "get-started/_index.md">}}">
      <h3>Quickstart Guide</h3>
      <p>
        A single page quickstart guide to get started with BadgerDB
      </p>
    </a>
  </div>
  <div class="item">
    <div class="icon"><i class="lni lni-book" aria-hidden="true"></i></div>
    <a href="{{< relref "resources/_index.md">}}">
      <h3>Resources</h3>
      <p>
        Additional resources and information
      </p>
    </a>
  </div>
  <div class="item">
    <div class="icon"><i class="lni lni-layers" aria-hidden="true"></i></div>
    <a href="{{< relref "design/_index.md">}}">
      <h3>Design</h3>
      <p>
        Design goals behind BadgerDB
      </p>
    </a>
  </div>

  <div class="item">
    <div class="icon"><i class="lni lni-direction-alt" aria-hidden="true"></i></div>
    <a href="{{< relref "projects-using-badger/_index.md">}}">
      <h3>Projects using Badger</h3>
      <p>
        A list of known projects that use BadgerDB
      </p>
    </a>
  </div>
  <div class="item">
    <div class="icon"><i class="lni lni-question-circle" aria-hidden="true"></i></div>
    <a href="{{< relref "faq/_index.md">}}">
      <h3>FAQ</h3>
      <p>
        Frequently asked questions
      </p>
    </a>
  </div>
  <div class="item">
    <div class="icon"><i class="lni lni-database" aria-hidden="true"></i></div>
    <a href="https://dgraph.io/badger">
      <h3>Badger</h3>
      <p>
        Embeddable, persistent, and fast key-value database that powers Dgraph
      </p>
    </a>
  </div>

</div>

<style>
  ul.contents {
    display: none;
  }
</style>

## Changelog

The [Changelog] is kept fairly up-to-date.

- Badger v1.0 was released in Nov 2017, and the latest version that is data-compatible
with v1.0 is v1.6.0.
- Badger v2.0 was released in Nov 2019 with a new storage format which won't
be compatible with all of the v1.x. Badger v2.0 supports compression, encryption and uses a cache to speed up lookup.

For more details on our version naming schema please read [Choosing a version]({{< relref "get-started/index.md#choosing-a-version" >}}).

[Changelog]:https://github.com/dgraph-io/badger/blob/master/CHANGELOG.md

## Contribute

<section class="toc">
  <div class="container">
    <div class="row row-no-padding">
      <div class="col-12 col-sm-6">
        <div class="section-item">
          <div class="section-name">
            <a href="https://github.com/dgraph-io/badger/blob/master/CONTRIBUTING.md">
              Contribute to Badger
            </a>
          </div>
          <p class="section-desc">
            Get started with contributing fixes and enhancements to Badger and related software.
          </p>
        </div>
      </div>
      </div>
  </div>
</section>

## Our Community

**Badger is made better every day by the growing community and the contributors all over the world.**

<section class="toc">
  <div class="container">
    <div class="row row-no-padding">
      <div class="col-12 col-sm-6">
        <div class="section-item">
          <div class="section-name">
            <a href="https://discuss.dgraph.io">
              Community
            </a>
          </div>
          <p class="section-desc">
            Discuss Badger on the official community.
          </p>
        </div>
      </div>
    </div>
  </div>
</section>
