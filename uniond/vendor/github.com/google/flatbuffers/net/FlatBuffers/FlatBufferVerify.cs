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
using System;
using System.Reflection;using System.Collections.Generic;
using System.IO;

namespace Google.FlatBuffers
{

  /// <summary>
  /// The Class of the Verifier Options
  /// </summary>
  public class Options
  {
    public const int DEFAULT_MAX_DEPTH = 64;
    public const int DEFAULT_MAX_TABLES = 1000000;

    private int max_depth = 0;
    private int max_tables = 0;
    private bool string_end_check = false;
    private bool alignment_check = false;

    public Options()
    {
      max_depth = DEFAULT_MAX_DEPTH;
      max_tables = DEFAULT_MAX_TABLES;
      string_end_check = true;
      alignment_check = true;
    }

    public Options(int maxDepth, int maxTables, bool stringEndCheck, bool alignmentCheck)
    {
      max_depth = maxDepth;
      max_tables = maxTables;
      string_end_check = stringEndCheck;
      alignment_check = alignmentCheck;
    }
    /// <summary> Maximum depth of nested tables allowed in a valid flatbuffer. </summary>
    public int maxDepth
    {
      get { return max_depth; }
      set { max_depth = value; }
    }
    /// <summary> Maximum number of tables allowed in a valid flatbuffer. </summary>
    public int maxTables
    {
      get { return max_tables; }
      set { max_tables = value; }
    }
    /// <summary> Check that string contains its null terminator </summary>
    public bool stringEndCheck
    {
      get { return string_end_check; }
      set { string_end_check = value; }
    }
    /// <summary> Check alignment of elements </summary>
    public bool alignmentCheck
    {
      get { return alignment_check; }
      set { alignment_check = value; }
    }
  }

  public struct checkElementStruct
  {
    public bool elementValid;
    public uint elementOffset;
  }

  public delegate bool VerifyTableAction(Verifier verifier, uint tablePos);
  public delegate bool VerifyUnionAction(Verifier verifier, byte typeId, uint tablePos);

  /// <summary>
  /// The Main Class of the FlatBuffer Verifier
  /// </summary>
  public class Verifier
  {
    private ByteBuffer verifier_buffer = null;
    private Options verifier_options = null;
    private int depth_cnt = 0;
    private int num_tables_cnt = 0;

    public const int SIZE_BYTE = 1;
    public const int SIZE_INT = 4;
    public const int SIZE_U_OFFSET = 4;
    public const int SIZE_S_OFFSET = 4;
    public const int SIZE_V_OFFSET = 2;
    public const int SIZE_PREFIX_LENGTH = FlatBufferConstants.SizePrefixLength;         // default size = 4
    public const int FLATBUFFERS_MAX_BUFFER_SIZE = System.Int32.MaxValue;               // default size = 2147483647
    public const int FILE_IDENTIFIER_LENGTH = FlatBufferConstants.FileIdentifierLength; // default size = 4

    /// <summary> The Base Constructor of the Verifier object </summary>
    public Verifier()
    {
      // Verifier buffer
      verifier_buffer = null;
      // Verifier settings 
      verifier_options = null;
      // Depth counter
      depth_cnt = 0;
      // Tables counter
      num_tables_cnt = 0;
    }

    /// <summary> The Constructor of the Verifier object with input parameters: ByteBuffer and/or Options </summary>
    /// <param name="buf"> Input flat byte buffer defined as ByteBuffer type</param>
    /// <param name="options"> Options object with settings for the coniguration the Verifier </param>
    public Verifier(ByteBuffer buf, Options options = null)
    {
      verifier_buffer = buf;
      verifier_options = options ?? new Options();
      depth_cnt = 0;
      num_tables_cnt = 0;
    }

    /// <summary> Bytes Buffer for Verify</summary>
    public ByteBuffer Buf
    {
      get { return verifier_buffer; }
      set { verifier_buffer = value; }
    }
    /// <summary> Options of the Verifier </summary>
    public Options options
    {
      get { return verifier_options; }
      set { verifier_options = value; }
    }
    /// <summary> Counter of tables depth in a tested flatbuffer  </summary>
    public int depth
    {
      get { return depth_cnt; }
      set { depth_cnt = value; }
    }
    /// <summary> Counter of tables in a tested flatbuffer </summary>
    public int numTables
    {
      get { return num_tables_cnt; }
      set { num_tables_cnt = value; }
    }


    /// <summary> Method set maximum tables depth of valid structure</summary>
    /// <param name="value"> Specify Value of the maximum depth of the structure</param>
    public Verifier SetMaxDepth(int value)
    {
      verifier_options.maxDepth = value;
      return this;
    }
    /// <summary> Specify maximum number of tables in structure </summary>
    /// <param name="value"> Specify Value of the maximum number of the tables in the structure</param>
    public Verifier SetMaxTables(int value)
    {
      verifier_options.maxTables = value;
      return this;
    }
    /// <summary> Enable/disable buffer content alignment check </summary>
    /// <param name="value"> Value of the State for buffer content alignment check (Enable = true) </param>
    public Verifier SetAlignmentCheck(bool value)
    {
      verifier_options.alignmentCheck = value;
      return this;
    }
    /// <summary> Enable/disable checking of string termination '0' character </summary>
    /// <param name="value"> Value of the option for string termination '0' character check (Enable = true)</param>
    public Verifier SetStringCheck(bool value)
    {
      verifier_options.stringEndCheck = value;
      return this;
    }

    /// <summary> Check if there is identifier in buffer </summary>
    /// <param name="buf"> Input flat byte buffer defined as ByteBuffer type </param>
    /// <param name="startPos">Start position of data in the Byte Buffer</param>
    /// <param name="identifier"> Identifier for the Byte Buffer</param>
    /// <returns> Return True when the Byte Buffer Identifier is present</returns>
    private bool BufferHasIdentifier(ByteBuffer buf, uint startPos, string identifier)
    {
      if (identifier.Length != FILE_IDENTIFIER_LENGTH)
      {
        throw new ArgumentException("FlatBuffers: file identifier must be length" + Convert.ToString(FILE_IDENTIFIER_LENGTH));
      }
      for (int i = 0; i < FILE_IDENTIFIER_LENGTH; i++)
      {
        if ((sbyte)identifier[i] != verifier_buffer.GetSbyte(Convert.ToInt32(SIZE_S_OFFSET + i + startPos)))
        {
          return false;
        }
      }

      return true;
    }

    /// <summary> Get UOffsetT from buffer at given position - it must be verified before read </summary>
    /// <param name="buf"> Input flat byte buffer defined as ByteBuffer type </param>
    /// <param name="pos"> Position of data in the Byte Buffer</param>
    /// <returns> Return the UOffset Value (Unsigned Integer type - 4 bytes) in pos </returns>
    private uint ReadUOffsetT(ByteBuffer buf, uint pos)
    {
      return buf.GetUint(Convert.ToInt32(pos));
    }
    /// <summary> Get SOffsetT from buffer at given position - it must be verified before read </summary>
    /// <param name="buf"> Input flat byte buffer defined as ByteBuffer type </param>
    /// <param name="pos"> Position of data in the Byte Buffer</param>
    /// <returns> Return the SOffset Value (Signed Integer type - 4 bytes) in pos </returns>
    private int ReadSOffsetT(ByteBuffer buf, int pos)
    {
      return buf.GetInt(pos);
    }
    /// <summary> Get VOffsetT from buffer at given position - it must be verified before read </summary>
    /// <param name="buf"> Input flat byte buffer defined as ByteBuffer type </param>
    /// <param name="pos"> Position of data in the Byte Buffer</param>
    /// <returns> Return the VOffset Value (Short type - 2 bytes) in pos </returns>
    private short ReadVOffsetT(ByteBuffer buf, int pos)
    {
      return buf.GetShort(pos);
    }

    /// <summary> Get table data area relative offset from vtable. Result is relative to table start
    ///           Fields which are deprecated are ignored by checking against the vtable's length. </summary>
    /// <param name="pos"> Position of data in the Byte Buffer </param>
    /// <param name="vtableOffset"> offset of value in the Table</param>
    /// <returns> Return the relative VOffset Value (Short type - 2 bytes) in calculated offset </returns>
    private short GetVRelOffset(int pos, short vtableOffset)
    {
      short VOffset = 0;
      // Used try/catch because pos typa as int 32bit
      try
      {
        // First, get vtable offset
        short vtable = Convert.ToInt16(pos - ReadSOffsetT(verifier_buffer, pos));
        // Check that offset points to vtable area (is smaller than vtable size)
        if (vtableOffset < ReadVOffsetT(verifier_buffer, vtable))
        {
          // Now, we can read offset value - TODO check this value against size of table data
          VOffset = ReadVOffsetT(verifier_buffer, vtable + vtableOffset);
        }
        else
        {
          VOffset = 0;
        }
      }
      catch (Exception e)
      {
        Console.WriteLine("Exception: {0}", e);
        return VOffset;
      }
      return VOffset;

    }
    /// <summary> Get table data area absolute offset from vtable. Result is the absolute buffer offset.
    /// The result value offset cannot be '0' (pointing to itself) so after validation this method returnes '0'
    /// value as a marker for missing optional entry </summary>
    /// <param name="tablePos"> Table Position value in the Byte Buffer </param>
    /// <param name="vtableOffset"> offset value in the Table</param>
    /// <returns> Return the absolute UOffset Value </returns>
    private uint GetVOffset(uint tablePos, short vtableOffset)
    {
      uint UOffset = 0;
      // First, get vtable relative offset
      short relPos = GetVRelOffset(Convert.ToInt32(tablePos), vtableOffset);
      if (relPos != 0)
      {
        // Calculate offset based on table postion
        UOffset = Convert.ToUInt32(tablePos + relPos);
      }
      else
      {
        UOffset = 0;
      }
      return UOffset;
    }

    /// <summary> Check flatbuffer complexity (tables depth, elements counter and so on) </summary>
    /// <returns> If complexity is too high function returns false as verification error </returns>
    private bool CheckComplexity()
    {
      return ((depth <= options.maxDepth) && (numTables <= options.maxTables));
    }

    /// <summary> Check alignment of element. </summary>
    /// <returns> Return True when alignment of the element is correct</returns>
    private bool CheckAlignment(uint element, ulong align)
    {
      return (((element & (align - 1)) == 0) || (!options.alignmentCheck));
    }

    /// <summary> Check if element is valid in buffer area. </summary> 
    /// <param name="pos"> Value defines the offset/position to element</param>
    /// <param name="elementSize"> Size of element</param>
    /// <returns> Return True when Element is correct </returns>
    private bool CheckElement(uint pos, ulong elementSize)
    {
      return ((elementSize < Convert.ToUInt64(verifier_buffer.Length)) && (pos <= (Convert.ToUInt32(verifier_buffer.Length) - elementSize)));
    }
    /// <summary> Check if element is a valid scalar. </summary>
    /// <param name="pos"> Value defines the offset to scalar</param>
    /// <param name="elementSize"> Size of element</param>
    /// <returns> Return True when Scalar Element is correct </returns>
    private bool CheckScalar(uint pos, ulong elementSize)
    {
      return ((CheckAlignment(pos, elementSize)) && (CheckElement(pos, elementSize)));
    }
    /// <summary> Check offset. It is a scalar with size of UOffsetT. </summary>
    private bool CheckOffset(uint offset)
    {
      return (CheckScalar(offset, SIZE_U_OFFSET));
    }

    private checkElementStruct CheckVectorOrString(uint pos, ulong elementSize)
    {
      var result = new checkElementStruct
      {
        elementValid = false,
        elementOffset = 0
      };

      uint vectorPos = pos;
      // Check we can read the vector/string size field (it is of uoffset size)
      if (!CheckScalar(vectorPos, SIZE_U_OFFSET))
      {
        // result.elementValid = false; result.elementOffset = 0;
        return result;
      }
      // Check the whole array. If this is a string, the byte past the array
      // must be 0.
      uint size = ReadUOffsetT(verifier_buffer, vectorPos);
      ulong max_elements = (FLATBUFFERS_MAX_BUFFER_SIZE / elementSize);
      if (size >= max_elements)
      {
        // Protect against byte_size overflowing. 
        // result.elementValid = false; result.elementOffset = 0;
        return result;
      }

      uint bytes_size = SIZE_U_OFFSET + (Convert.ToUInt32(elementSize) * size);
      uint buffer_end_pos = vectorPos + bytes_size;
      result.elementValid = CheckElement(vectorPos, bytes_size);
      result.elementOffset = buffer_end_pos;
      return (result);
    }

    /// <summary>Verify a string at given position.</summary>
    private bool CheckString(uint pos)
    {
      var result = CheckVectorOrString(pos, SIZE_BYTE);
      if (options.stringEndCheck)
      {
        result.elementValid = result.elementValid && CheckScalar(result.elementOffset, 1); // Must have terminator
        result.elementValid = result.elementValid && (verifier_buffer.GetSbyte(Convert.ToInt32(result.elementOffset)) == 0); // Terminating byte must be 0.
      }
      return result.elementValid;
    }

    /// <summary> Verify the vector of elements of given size </summary>
    private bool CheckVector(uint pos, ulong elementSize)
    {
      var result = CheckVectorOrString(pos, elementSize);
      return result.elementValid;
    }
    /// <summary> Verify table content using structure dependent generated function </summary>
    private bool CheckTable(uint tablePos, VerifyTableAction verifyAction)
    {
      return verifyAction(this, tablePos);
    }

    /// <summary> String check wrapper function to be used in vector of strings check </summary>
    private bool CheckStringFunc(Verifier verifier, uint pos)
    {
      return verifier.CheckString(pos);
    }

    /// <summary> Check vector of objects. Use generated object verification function </summary>
    private bool CheckVectorOfObjects(uint pos, VerifyTableAction verifyAction)
    {
      if (!CheckVector(pos, SIZE_U_OFFSET))
      {
        return false;
      }
      uint size = ReadUOffsetT(verifier_buffer, pos);
      // Vector data starts just after vector size/length
      uint vecStart = pos + SIZE_U_OFFSET;
      uint vecOff = 0;
      // Iterate offsets and verify referenced objects
      for (uint i = 0; i < size; i++)
      {
        vecOff = vecStart + (i * SIZE_U_OFFSET);
        if (!CheckIndirectOffset(vecOff))
        {
          return false;
        }
        uint objOffset = GetIndirectOffset(vecOff);
        if (!verifyAction(this, objOffset))
        {
          return false;
        }
      }
      return true;
    }

    /// <summary> Check if the offset referenced by offsetPos is the valid offset pointing to buffer</summary>
    //  offsetPos - offset to offset data
    private bool CheckIndirectOffset(uint pos)
    {
      // Check the input offset is valid
      if(!CheckScalar(pos, SIZE_U_OFFSET))
      {
        return false;
      }
      // Get indirect offset
      uint offset = ReadUOffsetT(verifier_buffer, pos);
      // May not point to itself neither wrap around  (buffers are max 2GB)
      if ((offset == 0) || (offset >= FLATBUFFERS_MAX_BUFFER_SIZE))
      {
        return false;
      }
      // Must be inside the buffer
      return CheckElement(pos + offset, 1);
    }

    /// <summary> Check flatbuffer content using generated object verification function </summary>
    private bool CheckBufferFromStart(string identifier, uint startPos, VerifyTableAction verifyAction)
    {
      if ((identifier != null) &&
          (identifier.Length == 0) &&
          ((verifier_buffer.Length < (SIZE_U_OFFSET + FILE_IDENTIFIER_LENGTH)) || (!BufferHasIdentifier(verifier_buffer, startPos, identifier))))
      {
        return false;
      }
      if(!CheckIndirectOffset(startPos))
      {
        return false;
      }
      uint offset = GetIndirectOffset(startPos);
      return CheckTable(offset, verifyAction); //  && GetComputedSize()
    }

    /// <summary> Get indirect offset. It is an offset referenced by offset Pos </summary>
    private uint GetIndirectOffset(uint pos)
    {
      // Get indirect offset referenced by offsetPos
      uint offset = pos + ReadUOffsetT(verifier_buffer, pos);
      return offset;
    }

    /// <summary> Verify beginning of table </summary>
    /// <param name="tablePos"> Position in the Table </param>
    /// <returns> Return True when the verification of the beginning of the table is passed</returns> 
    // (this method is used internally by generated verification functions)
    public bool VerifyTableStart(uint tablePos)
    {
      // Starting new table verification increases complexity of structure
      depth_cnt++;
      num_tables_cnt++;

      if (!CheckScalar(tablePos, SIZE_S_OFFSET))
      {
        return false;
      }
      uint vtable = (uint)(tablePos - ReadSOffsetT(verifier_buffer, Convert.ToInt32(tablePos)));
      return ((CheckComplexity()) && (CheckScalar(vtable, SIZE_V_OFFSET)) && (CheckAlignment(Convert.ToUInt32(ReadVOffsetT(verifier_buffer, Convert.ToInt32(vtable))), SIZE_V_OFFSET)) && (CheckElement(vtable, Convert.ToUInt64(ReadVOffsetT(verifier_buffer, Convert.ToInt32(vtable))))));
    }

    /// <summary> Verify end of table. In practice, this function does not check buffer but handles
    /// verification statistics update </summary>
    // (this method is used internally by generated verification functions)
    public bool VerifyTableEnd(uint tablePos)
    {
      depth--;
      return true;
    }

    /// <summary> Verifiy static/inlined data area field </summary>
    /// <param name="tablePos"> Position in the Table</param>
    /// <param name="offsetId"> Offset to the static/inlined data element </param>
    /// <param name="elementSize"> Size of the element </param>
    /// <param name="align"> Alignment bool value </param>
    /// <param name="required"> Required Value when the offset == 0 </param>
    /// <returns>Return True when the verification of the static/inlined data element is passed</returns>
    // (this method is used internally by generated verification functions)
    public bool VerifyField(uint tablePos, short offsetId, ulong elementSize, ulong align, bool required)
    {
      uint offset = GetVOffset(tablePos, offsetId);
      if (offset != 0)
      {
        return ((CheckAlignment(offset, align)) && (CheckElement(offset, elementSize)));
      }
      return !required; // it is OK if field is not required
    }

    /// <summary> Verify string </summary> 
    /// <param name="tablePos"> Position in the Table</param>
    /// <param name="vOffset"> Offset to the String element </param>
    /// <param name="required"> Required Value when the offset == 0 </param>
    /// <returns>Return True when the verification of the String is passed</returns>
    // (this method is used internally by generated verification functions)
    public bool VerifyString(uint tablePos, short vOffset, bool required)
    {
      var offset = GetVOffset(tablePos, vOffset);
      if (offset == 0)
      {
        return !required;
      }
      if (!CheckIndirectOffset(offset))
      {
        return false;
      }
      var strOffset = GetIndirectOffset(offset);
      return CheckString(strOffset);
    }

    /// <summary> Verify vector of fixed size structures and scalars </summary>
    /// <param name="tablePos"> Position in the Table</param>
    /// <param name="vOffset"> Offset to the Vector of Data </param>
    /// <param name="elementSize"> Size of the element</param>
    /// <param name="required"> Required Value when the offset == 0 </param>
    /// <returns>Return True when the verification of the Vector of Data passed</returns>
    // (this method is used internally by generated verification functions)
    public bool VerifyVectorOfData(uint tablePos, short vOffset, ulong elementSize, bool required)
    {
      var offset = GetVOffset(tablePos, vOffset);
      if (offset == 0)
      {
        return !required;
      }
      if (!CheckIndirectOffset(offset))
      {
        return false;
      }
      var vecOffset = GetIndirectOffset(offset);
      return  CheckVector(vecOffset, elementSize);
    }

    /// <summary> Verify array of strings </summary>
    /// <param name="tablePos"> Position in the Table</param>
    /// <param name="offsetId"> Offset to the Vector of String </param>
    /// <param name="required"> Required Value when the offset == 0 </param>
    /// <returns>Return True when the verification of the Vector of String passed</returns>
    // (this method is used internally by generated verification functions)
    public bool VerifyVectorOfStrings(uint tablePos, short offsetId, bool required)
    {
      var offset = GetVOffset(tablePos, offsetId);
      if (offset == 0)
      {
        return !required;
      }
      if (!CheckIndirectOffset(offset))
      {
        return false;
      }
      var vecOffset = GetIndirectOffset(offset);
      return CheckVectorOfObjects(vecOffset, CheckStringFunc); 
    }

    /// <summary> Verify vector of tables (objects). Tables are verified using generated verifyObjFunc </summary>
    /// <param name="tablePos"> Position in the Table</param>
    /// <param name="offsetId"> Offset to the Vector of Table </param>
    /// <param name="verifyAction"> Method used to the verification Table </param>
    /// <param name="required"> Required Value when the offset == 0 </param>
    /// <returns>Return True when the verification of the Vector of Table passed</returns>
    // (this method is used internally by generated verification functions)
    public bool VerifyVectorOfTables(uint tablePos, short offsetId, VerifyTableAction verifyAction, bool required)
    {
      var offset = GetVOffset(tablePos, offsetId);
      if (offset == 0)
      {
        return !required;
      }
      if (!CheckIndirectOffset(offset))
      {
        return false;
      }
      var vecOffset = GetIndirectOffset(offset);
      return CheckVectorOfObjects(vecOffset, verifyAction);
    }

    /// <summary> Verify table object using generated verification function. </summary>
    /// <param name="tablePos"> Position in the Table</param>
    /// <param name="offsetId"> Offset to the Table </param>
    /// <param name="verifyAction"> Method used to the verification Table </param>
    /// <param name="required"> Required Value when the offset == 0 </param>
    /// <returns>Return True when the verification of the Table passed</returns>
    // (this method is used internally by generated verification functions)
    public bool VerifyTable(uint tablePos, short offsetId, VerifyTableAction verifyAction, bool required)
    {
      var offset = GetVOffset(tablePos, offsetId);
      if (offset == 0)
      {
        return !required;
      }
      if (!CheckIndirectOffset(offset))
      {
        return false;
      }
      var tabOffset = GetIndirectOffset(offset);
      return CheckTable(tabOffset, verifyAction);
    }

    /// <summary> Verify nested buffer object. When verifyObjFunc is provided, it is used to verify object structure. </summary>
    /// <param name="tablePos"> Position in the Table </param>
    /// <param name="offsetId"> Offset to the Table </param>
    /// <param name="verifyAction">  Method used to the verification Table </param>
    /// <param name="required"> Required Value when the offset == 0  </param>
    // (this method is used internally by generated verification functions)
    public bool VerifyNestedBuffer(uint tablePos, short offsetId, VerifyTableAction verifyAction, bool required)
    {
      var offset = GetVOffset(tablePos, offsetId);
      if (offset == 0)
      {
        return !required;
      }
      uint vecOffset = GetIndirectOffset(offset);
      if (!CheckVector(vecOffset, SIZE_BYTE))
      {
        return false;
      }
      if (verifyAction != null)
      {
        var vecLength = ReadUOffsetT(verifier_buffer, vecOffset);
        // Buffer begins after vector length
        var vecStart = vecOffset + SIZE_U_OFFSET;
        // Create and Copy nested buffer bytes from part of Verify Buffer
        var nestedByteBuffer = new ByteBuffer(verifier_buffer.ToArray(Convert.ToInt32(vecStart), Convert.ToInt32(vecLength)));
        var nestedVerifyier = new Verifier(nestedByteBuffer, options);
        // There is no internal identifier - use empty one
        if (!nestedVerifyier.CheckBufferFromStart("", 0, verifyAction))
        {
          return false;
        }
      }
      return true;
    }

    /// <summary> Verifiy static/inlined data area at absolute offset </summary>
    /// <param name="pos"> Position of static/inlined data area in the Byte Buffer</param>
    /// <param name="elementSize"> Size of the union data</param>
    /// <param name="align"> Alignment bool value </param>
    /// <returns>Return True when the verification of the Union Data is passed</returns>
    // (this method is used internally by generated verification functions)
    public bool VerifyUnionData(uint pos, ulong elementSize, ulong align)
    {
      bool result = ((CheckAlignment(pos, align)) && (CheckElement(pos, elementSize)));
      return result;
    }

    /// <summary> Verify string referenced by absolute offset value </summary>
    /// <param name="pos"> Position of Union String in the Byte Buffer</param>
    /// <returns>Return True when the verification of the Union String is passed</returns> 
    // (this method is used internally by generated verification functions)
    public bool VerifyUnionString(uint pos)
    {
      bool result = CheckString(pos);
      return result;
    }

    /// <summary> Method verifies union object using generated verification function </summary>
    /// <param name="tablePos"> Position in the Table</param>
    /// <param name="typeIdVOffset"> Offset in the Table</param>
    /// <param name="valueVOffset"> Offset to Element</param>
    /// <param name="verifyAction"> Verification Method used for Union</param>
    /// <param name="required"> Required Value when the offset == 0 </param>
    // (this method is used internally by generated verification functions)
    public bool VerifyUnion(uint tablePos, short typeIdVOffset, short valueVOffset, VerifyUnionAction verifyAction, bool required)
    {
      // Check the union type index
      var offset = GetVOffset(tablePos, typeIdVOffset);
      if (offset == 0)
      {
        return !required;
      }
      if (!((CheckAlignment(offset, SIZE_BYTE)) && (CheckElement(offset, SIZE_BYTE))))
      {
        return false;
      }
      // Check union data
      offset = GetVOffset(tablePos, valueVOffset);
      // Take type id
      var typeId = verifier_buffer.Get(Convert.ToInt32(offset));
      if (offset == 0)
      {
        // When value data is not present, allow union verification function to deal with illegal offset
        return verifyAction(this, typeId, Convert.ToUInt32(verifier_buffer.Length));
      }
      if (!CheckIndirectOffset(offset))
      {
        return false;
      }
      // Take value offset and validate union structure
      uint unionOffset = GetIndirectOffset(offset);
      return verifyAction(this, typeId, unionOffset);
    }

    /// <summary> Verify vector of unions (objects). Unions are verified using generated verifyObjFunc </summary>
    /// <param name="tablePos"> Position of the Table</param>
    /// <param name="typeOffsetId"> Offset in the Table (Union type id)</param>
    /// <param name="offsetId"> Offset to vector of Data Stucture offset</param>
    /// <param name="verifyAction"> Verification Method used for Union</param>
    /// <param name="required"> Required Value when the offset == 0 </param>
    /// <returns>Return True when the verification of the Vector of Unions passed</returns>
    // (this method is used internally by generated verification functions)
    public bool VerifyVectorOfUnion(uint tablePos, short typeOffsetId, short offsetId, VerifyUnionAction verifyAction, bool required)
    {
      // type id offset must be valid
      var offset = GetVOffset(tablePos, typeOffsetId);
      if (offset == 0)
      {
        return !required;
      }
      if (!CheckIndirectOffset(offset))
      {
        return false;
      }
      // Get type id table absolute offset
      var typeIdVectorOffset = GetIndirectOffset(offset);
      // values offset must be valid
      offset = GetVOffset(tablePos, offsetId);
      if (!CheckIndirectOffset(offset))
      {
        return false;
      }
      var valueVectorOffset = GetIndirectOffset(offset);
      // validate referenced vectors
      if(!CheckVector(typeIdVectorOffset, SIZE_BYTE) ||
         !CheckVector(valueVectorOffset, SIZE_U_OFFSET))
      {
        return false;
      }
      // Both vectors should have the same length
      var typeIdVectorLength = ReadUOffsetT(verifier_buffer, typeIdVectorOffset);
      var valueVectorLength = ReadUOffsetT(verifier_buffer, valueVectorOffset);
      if (typeIdVectorLength != valueVectorLength)
      {
        return false;
      }
      // Verify each union from vectors
      var typeIdStart = typeIdVectorOffset + SIZE_U_OFFSET;
      var valueStart = valueVectorOffset + SIZE_U_OFFSET;
      for (uint i = 0; i < typeIdVectorLength; i++)
      {
        // Get type id
        byte typeId = verifier_buffer.Get(Convert.ToInt32(typeIdStart + i * SIZE_U_OFFSET));
        // get offset to vector item
        uint off = valueStart + i * SIZE_U_OFFSET;
        // Check the vector item has a proper offset
        if (!CheckIndirectOffset(off))
        {
          return false;
        }
        uint valueOffset = GetIndirectOffset(off);
        // Verify object
        if (!verifyAction(this, typeId, valueOffset))
        {
          return false;
        }
      }
      return true;
    }

    // Method verifies flatbuffer data using generated Table verification function.
    // The data buffer is already provided when creating [Verifier] object (see [NewVerifier])
    //
    //   - identifier - the expected identifier of buffer data.
    //     When empty identifier is provided the identifier validation is skipped.
    //   - sizePrefixed - this flag should be true when buffer is prefixed with content size
    //   - verifyObjFunc - function to be used for verification. This function is generated by compiler and included in each table definition file with name "<Tablename>Verify"
    //
    // Example:
    //
    //  /* Verify Monster table. Ignore buffer name and assume buffer does not contain data length prefix */
    //  isValid = verifier.verifyBuffer(bb, false, MonsterVerify)
    //
    //  /* Verify Monster table. Buffer name is 'MONS' and contains data length prefix */
    //  isValid = verifier.verifyBuffer("MONS", true, MonsterVerify)
    /// <summary> Method verifies flatbuffer data using generated Table verification function </summary>
    /// 
    /// <param name="identifier"> The expected identifier of buffer data</param>
    /// <param name="sizePrefixed"> Flag should be true when buffer is prefixed with content size</param>
    /// <param name="verifyAction"> Function to be used for verification. This function is generated by compiler and included in each table definition file</param>
    /// <returns> Return True when verification of FlatBuffer passed</returns>
    /// <example>
    /// Example 1. Verify Monster table. Ignore buffer name and assume buffer does not contain data length prefix 
    /// <code>  isValid = verifier.VerifyBuffer(bb, false, MonsterVerify)</code>
    /// Example 2. Verify Monster table. Buffer name is 'MONS' and contains data length prefix 
    /// <code>  isValid = verifier.VerifyBuffer("MONS", true, MonsterVerify)</code>
    /// </example>
    public bool VerifyBuffer(string identifier, bool sizePrefixed, VerifyTableAction verifyAction)
    {
      // Reset counters - starting verification from beginning
      depth = 0;
      numTables = 0;

      var start = (uint)(verifier_buffer.Position);
      if (sizePrefixed) 
      {
        start = (uint)(verifier_buffer.Position) + SIZE_PREFIX_LENGTH;
        if(!CheckScalar((uint)(verifier_buffer.Position), SIZE_PREFIX_LENGTH))
        {
          return false;
        }
        uint size = ReadUOffsetT(verifier_buffer, (uint)(verifier_buffer.Position));
        if (size != ((uint)(verifier_buffer.Length) - start))
        {
          return false;
        }
      } 
      return CheckBufferFromStart(identifier, start, verifyAction);
    }
  }

}
