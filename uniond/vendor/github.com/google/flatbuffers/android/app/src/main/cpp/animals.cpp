/*
 * Copyright 2014 Google Inc. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include <jni.h>
#include <string>
#include <search.h>
#include "generated/animal_generated.h"

using namespace com::fbs::app;
using namespace flatbuffers;

extern "C" JNIEXPORT jbyteArray JNICALL Java_com_flatbuffers_app_MainActivity_createAnimalFromJNI(
        JNIEnv* env,
        jobject /* this */) {
    // create a new animal flatbuffers
    auto fb = FlatBufferBuilder(1024);
    auto tiger = CreateAnimalDirect(fb, "Tiger", "Roar", 300);
    fb.Finish(tiger);

    // copies it to a Java byte array.
    auto buf = reinterpret_cast<jbyte*>(fb.GetBufferPointer());
    int size = fb.GetSize();
    auto ret = env->NewByteArray(size);
    env->SetByteArrayRegion (ret, 0, fb.GetSize(), buf);
  return ret;
}
