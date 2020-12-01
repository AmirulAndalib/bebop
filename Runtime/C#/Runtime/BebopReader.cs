﻿using System;
using System.Collections.Immutable;
using System.Runtime.CompilerServices;
using System.Text;
using Bebop.Exceptions;
using static System.Runtime.CompilerServices.Unsafe;
using static System.Runtime.InteropServices.MemoryMarshal;

namespace Bebop.Runtime
{
    public ref struct BebopReader
    {
        // ReSharper disable once InconsistentNaming
        private static readonly UTF8Encoding UTF8 = new();

        /// <summary>
        ///     A contiguous region of memory that contains the contents of a Bebop message
        /// </summary>
        // ReSharper disable once FieldCanBeMadeReadOnly.Local
        private ReadOnlySpan<byte> _buffer;

        /// <summary>
        /// 
        /// </summary>
        public int Position { get; set; }


        /// <summary>
        /// 
        /// </summary>
        /// <param name="buffer"></param>
        /// <returns></returns>
        [MethodImpl(BebopConstants.HotPath)]
        public static BebopReader From(ReadOnlySpan<byte> buffer) => new(buffer);

        [MethodImpl(BebopConstants.HotPath)]
        public static BebopReader From(byte[] buffer) => new(buffer);

        [MethodImpl(BebopConstants.HotPath)]
        public static BebopReader From(ReadOnlyMemory<byte> buffer) => new(buffer.Span);

        [MethodImpl(BebopConstants.HotPath)]
        public static BebopReader From(ImmutableArray<byte> buffer) => new(buffer.AsSpan());

        [MethodImpl(BebopConstants.HotPath)]
        public static BebopReader From(ArraySegment<byte> buffer) => new(buffer);

        private BebopReader(ReadOnlySpan<byte> buffer)
        {
            if (!BitConverter.IsLittleEndian)
            {
                throw new BebopViewException("Big-endian systems are not supported by Bebop.");
            }
            _buffer = buffer;
            Position = 0;
        }

        [MethodImpl(BebopConstants.HotPath)]
        public byte ReadByte() => _buffer[Position++];

        [MethodImpl(BebopConstants.HotPath)]
        public ushort ReadUInt16()
        {
            const int size = 2;
            var value = ReadUnaligned<ushort>(ref GetReference(_buffer.Slice(Position, size)));
            Position += size;
            return value;
        }

        [MethodImpl(BebopConstants.HotPath)]
        public short ReadInt16()
        {
            const int size = 2;
            var value = ReadUnaligned<short>(ref GetReference(_buffer.Slice(Position, size)));
            Position += size;
            return value;
        }

        [MethodImpl(BebopConstants.HotPath)]
        public uint ReadUInt32()
        {
            const int size = 4;
            var value = ReadUnaligned<uint>(ref GetReference(_buffer.Slice(Position, size)));
            Position += size;
            return value;
        }

        [MethodImpl(BebopConstants.HotPath)]
        public int ReadInt32()
        {
            const int size = 4;
            var value = ReadUnaligned<int>(ref GetReference(_buffer.Slice(Position, size)));
            Position += size;
            return value;
        }

        [MethodImpl(BebopConstants.HotPath)]
        public ulong ReadUInt64()
        {
            const int size = 8;
            var value = ReadUnaligned<ulong>(ref GetReference(_buffer.Slice(Position, size)));
            Position += size;
            return value;
        }

        [MethodImpl(BebopConstants.HotPath)]
        public long ReadInt64()
        {
            const int size = 8;
            var value = ReadUnaligned<long>(ref GetReference(_buffer.Slice(Position, size)));
            Position += size;
            return value;
        }

        [MethodImpl(BebopConstants.HotPath)]
        public float ReadFloat32()
        {
            const int size = 4;
            var value = ReadUnaligned<float>(ref GetReference(_buffer.Slice(Position, size)));
            Position += size;
            return value;
        }

        [MethodImpl(BebopConstants.HotPath)]
        public double ReadFloat64()
        {
            const int size = 8;
            var value = ReadUnaligned<double>(ref GetReference(_buffer.Slice(Position, size)));
            Position += size;
            return value;
        }

        [MethodImpl(BebopConstants.HotPath)]
        private static T ConvertFrom<T>(uint constant) where T : struct, Enum =>
            As<uint, T>(ref constant);

        [MethodImpl(BebopConstants.HotPath)]
        public T ReadEnum<T>() where T : struct, Enum => ConvertFrom<T>(ReadUInt32());

        [MethodImpl(BebopConstants.HotPath)]
        public DateTime ReadDate() => DateTime.FromBinary(ReadInt64());

        /// <summary>
        ///     Reads a null-terminated string from the underlying buffer
        /// </summary>
        /// <returns>A UTF-16 encoded string</returns>
        [MethodImpl(BebopConstants.HotPath)]
        public string ReadString()
        {
            var stringByteCount = unchecked((int) ReadUInt32());
            var stringSpan = _buffer.Slice(Position, stringByteCount);
            Position += stringByteCount;


        #if AGGRESSIVE_OPTIMIZE
            return UTF8.GetString(stringSpan);
        #else
            unsafe
            {
                fixed (byte* bytePtr = stringSpan)
                {
                    return UTF8.GetString(bytePtr, stringByteCount);
                }
            }
        #endif
        }

        /// <summary>
        ///     Reads a <see cref="Guid"/> from the underlying buffer and advances the advances the current position by 16 bytes.
        /// </summary>
        /// <returns>A well-formed Guid structure instances.</returns>
        [MethodImpl(BebopConstants.HotPath)]
        public Guid ReadGuid()
        {
            const int size = 16;
            var index = Position;
            Position += size;
            return ReadUnaligned<Guid>(ref GetReference(_buffer.Slice(index, size)));
        }

        /// <summary>
        /// Converts an array into a <see cref="ImmutableArray{T}"/> without copying
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="array"></param>
        /// <returns></returns>
        [MethodImpl(BebopConstants.HotPath)]
        private static ImmutableArray<T> AsImmutable<T>(T[] array) => As<T[], ImmutableArray<T>>(ref array);

        /// <summary>
        ///     Reads a length-prefixed byte array from the buffer
        /// </summary>
        /// <returns>An array of bytes</returns>
        [MethodImpl(BebopConstants.HotPath)]
        public ImmutableArray<byte> ReadBytes()
        {
            var length = unchecked((int) ReadUInt32());
            if (length == 0)
            {
                return ImmutableArray<byte>.Empty;
            }
            var index = Position;
            Position += length;
            var data = _buffer.Slice(index, length).ToArray();
            return As<byte[], ImmutableArray<byte>>(ref data);
        }

        /// <summary>
        ///     Read out a message's length prefix.
        /// </summary>
        [MethodImpl(BebopConstants.HotPath)]
        public uint ReadRecordLength()
        {
            const int size = 4;
            var result = ReadUnaligned<uint>(ref GetReference(_buffer.Slice(Position, size)));
            Position += size;
            return result;
        }
    }
}