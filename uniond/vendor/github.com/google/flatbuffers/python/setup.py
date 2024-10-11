# Copyright 2016 Google Inc. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

from setuptools import setup

setup(
    name='flatbuffers',
    version='24.3.25',
    license='Apache 2.0',
    license_files='../LICENSE',
    author='Derek Bailey',
    author_email='derekbailey@google.com',
    url='https://google.github.io/flatbuffers/',
    long_description=('Python runtime library for use with the '
                      '`Flatbuffers <https://google.github.io/flatbuffers/>`_ '
                      'serialization format.'),
    packages=['flatbuffers'],
    include_package_data=True,
    requires=[],
    description='The FlatBuffers serialization format for Python',
    classifiers=[
        'Intended Audience :: Developers',
        'License :: OSI Approved :: Apache Software License',
        'Operating System :: OS Independent',
        'Programming Language :: Python',
        'Programming Language :: Python :: 2',
        'Programming Language :: Python :: 3',
        'Topic :: Software Development :: Libraries :: Python Modules',
    ],
    project_urls={
        'Documentation': 'https://google.github.io/flatbuffers/',
        'Source': 'https://github.com/google/flatbuffers',
    },
)
