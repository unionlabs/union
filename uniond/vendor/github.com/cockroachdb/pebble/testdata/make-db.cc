// Copyright 2012 The LevelDB-Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// This program creates a leveldb db at /tmp/db.
//
// To build and run:
// g++ make-db.cc -lleveldb && ./a.out

#include <iostream>
#include <sstream>

#include "rocksdb/db.h"

template <typename T>
inline std::string ToString(const T& t) {
  std::ostringstream s;
  s << t;
  return s.str();
}

template <typename T>
inline T FromString(const std::string &str, T val = T()) {
  std::istringstream s(str);
  s >> val;
  return val;
}

int main(int argc, char** argv) {
  rocksdb::Status status;
  rocksdb::Options o;
  rocksdb::WriteOptions wo;
  rocksdb::DB* db;

  o.create_if_missing = true;
  o.error_if_exists = true;

  // The program consists of up to 4 stages. If stage is in the range [1, 4],
  // the program will exit after the stage'th stage.
  // 1. create an empty DB.
  // 2. add some key/value pairs.
  // 3. close and re-open the DB, which forces a compaction.
  // 4. add some more key/value pairs.
  if (argc != 2) {
    std::cerr << "usage: " << argv[0] << " [1,2,3,4]\n";
    return 1;
  }

  const int stage = FromString<int>(argv[1]);
  if (stage < 1) {
    return 0;
  }
  std::cout << "Stage 1" << std::endl;

  const std::string dbname = "db-stage-" + ToString(stage);
  status = rocksdb::DB::Open(o, dbname, &db);
  if (!status.ok()) {
    std::cerr << "DB::Open " << status.ToString() << std::endl;
    return 1;
  }

  if (stage < 2) {
    return 0;
  }
  std::cout << "Stage 2" << std::endl;

  status = db->Put(wo, "foo", "one");
  if (!status.ok()) {
    std::cerr << "DB::Put " << status.ToString() << std::endl;
    return 1;
  }

  status = db->Put(wo, "bar", "two");
  if (!status.ok()) {
    std::cerr << "DB::Put " << status.ToString() << std::endl;
    return 1;
  }

  status = db->Put(wo, "baz", "three");
  if (!status.ok()) {
    std::cerr << "DB::Put " << status.ToString() << std::endl;
    return 1;
  }

  status = db->Put(wo, "foo", "four");
  if (!status.ok()) {
    std::cerr << "DB::Put " << status.ToString() << std::endl;
    return 1;
  }

  status = db->Delete(wo, "bar");
  if (!status.ok()) {
    std::cerr << "DB::Delete " << status.ToString() << std::endl;
    return 1;
  }

  if (stage < 3) {
    return 0;
  }
  std::cout << "Stage 3" << std::endl;

  delete db;
  db = NULL;
  o.create_if_missing = false;
  o.error_if_exists = false;

  status = rocksdb::DB::Open(o, dbname, &db);
  if (!status.ok()) {
    std::cerr << "DB::Open " << status.ToString() << std::endl;
    return 1;
  }

  if (stage < 4) {
    return 0;
  }
  std::cout << "Stage 4" << std::endl;

  status = db->Put(wo, "foo", "five");
  if (!status.ok()) {
    std::cerr << "DB::Put " << status.ToString() << std::endl;
    return 1;
  }

  status = db->Put(wo, "quux", "six");
  if (!status.ok()) {
    std::cerr << "DB::Put " << status.ToString() << std::endl;
    return 1;
  }

  status = db->Delete(wo, "baz");
  if (!status.ok()) {
    std::cerr << "DB::Delete " << status.ToString() << std::endl;
    return 1;
  }

  return 0;
}
