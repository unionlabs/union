// Copyright 2011 The LevelDB-Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// This program adds N lines from infile to a leveldb table at outfile.
// The h.txt infile was generated via:
// cat hamlet-act-1.txt | tr '[:upper:]' '[:lower:]' | grep -o -E '\w+' | sort | uniq -c > infile
//
// To build and run:
// g++ make-table.cc -lleveldb && ./a.out

#include <fstream>
#include <iostream>
#include <string>

#include "rocksdb/env.h"
#include "rocksdb/filter_policy.h"
#include "rocksdb/slice_transform.h"
#include "rocksdb/sst_file_writer.h"
#include "rocksdb/table.h"

const char* infile = "h.txt";

// A dummy prefix extractor that cuts off the last two bytes for keys of
// length three or over. This is not a valid prefix extractor and barely
// enough to do a little bit of unit testing.
//
// TODO(tbg): write some test infra using CockroachDB MVCC data.
class PrefixExtractor : public rocksdb::SliceTransform {
 public:
  PrefixExtractor() {}

  virtual const char* Name() const { return "leveldb.BytewiseComparator"; }

  virtual rocksdb::Slice Transform(const rocksdb::Slice& src) const {
    auto sl = rocksdb::Slice(src.data(), src.size());
    return sl;
  }

  virtual bool InDomain(const rocksdb::Slice& src) const { return true; }
};

class KeyCountPropertyCollector : public rocksdb::TablePropertiesCollector {
 public:
  KeyCountPropertyCollector()
      : count_(0) {
  }

  rocksdb::Status AddUserKey(const rocksdb::Slice&, const rocksdb::Slice&,
                             rocksdb::EntryType type, rocksdb::SequenceNumber,
                             uint64_t) override {
    count_++;
    return rocksdb::Status::OK();
  }

  rocksdb::Status Finish(rocksdb::UserCollectedProperties* properties) override {
    char buf[16];
    sprintf(buf, "%d", count_);
    *properties = rocksdb::UserCollectedProperties{
      {"test.key-count", buf},
    };
    return rocksdb::Status::OK();
  }

  const char* Name() const override { return "KeyCountPropertyCollector"; }

  rocksdb::UserCollectedProperties GetReadableProperties() const override {
    return rocksdb::UserCollectedProperties{};
  }

 private:
  int count_;
};

class KeyCountPropertyCollectorFactory : public rocksdb::TablePropertiesCollectorFactory {
  virtual rocksdb::TablePropertiesCollector* CreateTablePropertiesCollector(
      rocksdb::TablePropertiesCollectorFactory::Context context) override {
    return new KeyCountPropertyCollector();
  }
  const char* Name() const override { return "KeyCountPropertyCollector"; }
};

int write() {
  for (int i = 0; i < 12; ++i) {
    rocksdb::Options options;
    rocksdb::BlockBasedTableOptions table_options;
    const char* outfile;

    table_options.block_size = 2048;
    table_options.index_shortening = rocksdb::BlockBasedTableOptions::IndexShorteningMode::kShortenSeparatorsAndSuccessor;

    switch (i) {
      case 0:
        outfile = "h.ldb";
        table_options.format_version = 0;
        table_options.whole_key_filtering = false;
        break;

      case 1:
        outfile = "h.sst";
        options.table_properties_collector_factories.emplace_back(
            new KeyCountPropertyCollectorFactory);
        table_options.whole_key_filtering = false;
        break;

      case 2:
        outfile = "h.no-compression.sst";
        options.table_properties_collector_factories.emplace_back(
            new KeyCountPropertyCollectorFactory);
        options.compression = rocksdb::kNoCompression;
        table_options.whole_key_filtering = false;
        break;

      case 3:
        outfile = "h.block-bloom.no-compression.sst";
        options.compression = rocksdb::kNoCompression;
        table_options.filter_policy.reset(rocksdb::NewBloomFilterPolicy(10, true));
        table_options.whole_key_filtering = true;
        break;

      case 4:
        outfile = "h.table-bloom.sst";
        table_options.filter_policy.reset(rocksdb::NewBloomFilterPolicy(10, false));
        table_options.whole_key_filtering = true;
        break;

      case 5:
        outfile = "h.table-bloom.no-compression.sst";
        options.compression = rocksdb::kNoCompression;
        table_options.filter_policy.reset(rocksdb::NewBloomFilterPolicy(10, false));
        table_options.whole_key_filtering = true;
        break;

      case 6:
        // TODO(peter): unused at this time
        //
        // outfile = "h.block-bloom.no-compression.prefix_extractor.sst";
        // options.compression = rocksdb::kNoCompression;
        // options.prefix_extractor.reset(new PrefixExtractor);
        // table_options.filter_policy.reset(rocksdb::NewBloomFilterPolicy(10, true));
        // table_options.whole_key_filtering = true;
        // break;
        continue;

      case 7:
        // TODO(peter): unused at this time
        //
        // outfile = "h.table-bloom.no-compression.prefix_extractor.sst";
        // options.compression = rocksdb::kNoCompression;
        // options.prefix_extractor.reset(new PrefixExtractor);
        // table_options.filter_policy.reset(rocksdb::NewBloomFilterPolicy(10, false));
        // table_options.whole_key_filtering = true;
        // break;
        continue;

      case 8:
        // TODO(peter): unused at this time
        //
        // outfile = "h.block-bloom.no-compression.prefix_extractor.no_whole_key_filter.sst";
        // options.compression = rocksdb::kNoCompression;
        // options.prefix_extractor.reset(new PrefixExtractor);
        // table_options.filter_policy.reset(rocksdb::NewBloomFilterPolicy(10, true));
        // table_options.whole_key_filtering = false;
        // break;
        continue;

      case 9:
        outfile = "h.table-bloom.no-compression.prefix_extractor.no_whole_key_filter.sst";
        options.compression = rocksdb::kNoCompression;
        options.prefix_extractor.reset(new PrefixExtractor);
        table_options.filter_policy.reset(rocksdb::NewBloomFilterPolicy(10, false));
        table_options.whole_key_filtering = false;
        break;

      case 10:
        outfile = "h.no-compression.two_level_index.sst";
        options.table_properties_collector_factories.emplace_back(
            new KeyCountPropertyCollectorFactory);
        options.compression = rocksdb::kNoCompression;
        table_options.index_type = rocksdb::BlockBasedTableOptions::IndexType::kTwoLevelIndexSearch;
        // Use small metadata_block_size to stress two_level_index.
        table_options.metadata_block_size = 128;
        table_options.whole_key_filtering = false;
        break;

      case 11:
        outfile = "h.zstd-compression.sst";
        options.table_properties_collector_factories.emplace_back(
            new KeyCountPropertyCollectorFactory);
        options.compression = rocksdb::kZSTD;
        table_options.whole_key_filtering = false;
        break;

      default:
        continue;
    }

    options.table_factory.reset(rocksdb::NewBlockBasedTableFactory(table_options));

    std::unique_ptr<rocksdb::SstFileWriter> tb(new rocksdb::SstFileWriter({}, options));
    rocksdb::Status status = tb->Open(outfile);
    if (!status.ok()) {
      std::cerr << "SstFileWriter::Open: " << status.ToString() << std::endl;
      return 1;
    }

    int rangeDelLength = 0;
    int rangeDelCounter = 0;
    std::ifstream in(infile);
    std::string s;
    std::string rangeDelStart;
    for (int i = 0; getline(in, s); i++) {
      std::string key(s, 8);
      std::string val(s, 0, 7);
      val = val.substr(1 + val.rfind(' '));
      tb->Put(key.c_str(), val.c_str());
      // Add range deletions of increasing length.
      if (i % 100 == 0) {
        rangeDelStart = key;
        rangeDelCounter = 0;
        rangeDelLength++;
      }
      rangeDelCounter++;

      if (rangeDelCounter == rangeDelLength) {
        tb->DeleteRange(rangeDelStart, key.c_str());
      }
    }

    rocksdb::ExternalSstFileInfo info;
    status = tb->Finish(&info);
    if (!status.ok()) {
      std::cerr << "TableBuilder::Finish: " << status.ToString() << std::endl;
      return 1;
    }

    std::cout << outfile << ": wrote " << info.num_entries << " entries, " << info.file_size << "b" << std::endl;
  }
  return 0;
}

int main(int argc, char** argv) {
  return write();

}
