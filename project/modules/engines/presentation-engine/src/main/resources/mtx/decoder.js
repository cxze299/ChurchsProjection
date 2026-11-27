// mtx-decompressor 1.9.0, Mozilla Public License 2.0. See LICENSE in this directory.
// Source: https://github.com/ChristopherVR/mtx-decompressor
// The npm dist/index.js was transpiled with TypeScript 5.9.3 using
// --allowJs --target ES5 --downlevelIteration --module commonjs.
var __extends = (this && this.__extends) || (function () {
    var extendStatics = function (d, b) {
        extendStatics = Object.setPrototypeOf ||
            ({ __proto__: [] } instanceof Array && function (d, b) { d.__proto__ = b; }) ||
            function (d, b) { for (var p in b) if (Object.prototype.hasOwnProperty.call(b, p)) d[p] = b[p]; };
        return extendStatics(d, b);
    };
    return function (d, b) {
        if (typeof b !== "function" && b !== null)
            throw new TypeError("Class extends value " + String(b) + " is not a constructor or null");
        extendStatics(d, b);
        function __() { this.constructor = d; }
        d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
    };
})();
var __assign = (this && this.__assign) || function () {
    __assign = Object.assign || function(t) {
        for (var s, i = 1, n = arguments.length; i < n; i++) {
            s = arguments[i];
            for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p))
                t[p] = s[p];
        }
        return t;
    };
    return __assign.apply(this, arguments);
};
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
var __read = (this && this.__read) || function (o, n) {
    var m = typeof Symbol === "function" && o[Symbol.iterator];
    if (!m) return o;
    var i = m.call(o), r, ar = [], e;
    try {
        while ((n === void 0 || n-- > 0) && !(r = i.next()).done) ar.push(r.value);
    }
    catch (error) { e = { error: error }; }
    finally {
        try {
            if (r && !r.done && (m = i["return"])) m.call(i);
        }
        finally { if (e) throw e.error; }
    }
    return ar;
};
var __spreadArray = (this && this.__spreadArray) || function (to, from, pack) {
    if (pack || arguments.length === 2) for (var i = 0, l = from.length, ar; i < l; i++) {
        if (ar || !(i in from)) {
            if (!ar) ar = Array.prototype.slice.call(from, 0, i);
            ar[i] = from[i];
        }
    }
    return to.concat(ar || Array.prototype.slice.call(from));
};
var __values = (this && this.__values) || function(o) {
    var s = typeof Symbol === "function" && Symbol.iterator, m = s && o[s], i = 0;
    if (m) return m.call(o);
    if (o && typeof o.length === "number") return {
        next: function () {
            if (o && i >= o.length) o = void 0;
            return { value: o && o[i++], done: !o };
        }
    };
    throw new TypeError(s ? "Object is not iterable." : "Symbol.iterator is not defined.");
};
var _a;
Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });
//#region src/errors.ts
/**
* Structured error type for the MTX/EOT decoder.
*
* Mirrors libeot's `enum EOTError` (MPL 2.0 — inc/libeot/EOTError.h) so that
* failures are machine-discriminable via a stable `code` rather than only a
* human-readable message. Codes at or above {@link EOT_WARN} denote recoverable
* warnings ("the font is usable, but…") as opposed to fatal errors.
*/
/** Threshold at or above which a code is a non-fatal warning. */
var EOT_WARN = 1e3;
/** Discriminable error/warning codes, mirroring libeot's `enum EOTError`. */
var EotErrorCode = /* @__PURE__ */ function (EotErrorCode) {
    EotErrorCode["InsufficientBytes"] = "INSUFFICIENT_BYTES";
    EotErrorCode["HeaderTooBig"] = "HEADER_TOO_BIG";
    EotErrorCode["BogusStringSize"] = "BOGUS_STRING_SIZE";
    EotErrorCode["CorruptFile"] = "CORRUPT_FILE";
    EotErrorCode["LogicError"] = "LOGIC_ERROR";
    EotErrorCode["NoMaxpTable"] = "NO_MAXP_TABLE";
    EotErrorCode["NoHeadTable"] = "NO_HEAD_TABLE";
    EotErrorCode["NoHmtxTable"] = "NO_HMTX_TABLE";
    EotErrorCode["CorruptHopcodeData"] = "CORRUPT_HOPCODE_DATA";
    EotErrorCode["MalformedHeadTable"] = "MALFORMED_HEAD_TABLE";
    /** A byte-level read/write/seek was attempted while mid-byte (`bitPos != 0`). */
    EotErrorCode["OffByteBoundary"] = "OFF_BYTE_BOUNDARY";
    /** A write or copy would exceed the stream's reserved capacity. */
    EotErrorCode["OutOfReservedSpace"] = "OUT_OF_RESERVED_SPACE";
    /** A seek would move past the stream's reserved end. */
    EotErrorCode["SeekPastEos"] = "SEEK_PAST_EOS";
    EotErrorCode["MtxError"] = "MTX_ERROR";
    /** Recoverable: the coded version was wrong but a retry succeeded. */
    EotErrorCode["WarnBadVersion"] = "WARN_BAD_VERSION";
    /** Recoverable: uncompressed font data lacks a recognised sfnt signature. */
    EotErrorCode["WarnNotSfnt"] = "WARN_NOT_SFNT";
    /** Recoverable: `maxp` is not version 1.0 (TrueType). */
    EotErrorCode["WarnMaxpVersion"] = "WARN_MAXP_VERSION";
    /** Recoverable: a recommended table is absent from the font. */
    EotErrorCode["WarnMissingTable"] = "WARN_MISSING_TABLE";
    return EotErrorCode;
}({});
/** Numeric tier per code — warnings sort at/above {@link EOT_WARN}. */
var WARNING_CODES = /* @__PURE__ */ new Set([
    "WARN_BAD_VERSION",
    "WARN_NOT_SFNT",
    "WARN_MAXP_VERSION",
    "WARN_MISSING_TABLE"
]);
/** An error (or warning) raised while decoding MTX/EOT font data. */
var EotError = /** @class */ (function (_super) {
    __extends(EotError, _super);
    function EotError(code, message, options) {
        var _this = _super.call(this, message, options) || this;
        _this.name = "EotError";
        _this.code = code;
        Object.setPrototypeOf(_this, EotError.prototype);
        return _this;
    }
    Object.defineProperty(EotError.prototype, "isWarning", {
        /** True when this represents a recoverable warning rather than a fatal error. */
        get: function () {
            return WARNING_CODES.has(this.code);
        },
        enumerable: false,
        configurable: true
    });
    return EotError;
}(Error));
/** Rethrow anything that is not an {@link EotError} as a `CorruptFile` EotError, keeping the cause. */
function toEotError(err) {
    if (err instanceof EotError)
        return err;
    return new EotError("CORRUPT_FILE", "malformed font data: ".concat(err instanceof Error ? err.message : String(err)), { cause: err });
}
/** Dispatch a warning to whichever sinks are set. */
function emitWarning(sinks, code, message) {
    var _b, _c;
    (_b = sinks === null || sinks === void 0 ? void 0 : sinks.onWarn) === null || _b === void 0 ? void 0 : _b.call(sinks, message);
    (_c = sinks === null || sinks === void 0 ? void 0 : sinks.onWarning) === null || _c === void 0 ? void 0 : _c.call(sinks, {
        code: code,
        message: message
    });
}
//#endregion
//#region src/stream.ts
/**
* Binary stream reader/writer for big-endian data.
* Ported from libeot (MPL 2.0) util/stream.c
*/
var Stream = /** @class */ (function () {
    function Stream(buf, size) {
        if (buf) {
            this.buf = buf;
            this.size = size;
            this.reserved = buf.length;
        }
        else {
            this.buf = /* @__PURE__ */ new Uint8Array(0);
            this.size = 0;
            this.reserved = 0;
        }
        this.pos = 0;
        this.bitPos = 0;
    }
    Stream.fromExisting = function (buf, size, reserved) {
        var s = new Stream(null, 0);
        s.buf = buf;
        s.size = size;
        s.reserved = reserved;
        return s;
    };
    Stream.prototype.reserve = function (n) {
        if (this.reserved >= n)
            return;
        var newBuf = new Uint8Array(Math.max(n, this.reserved * 2));
        newBuf.set(this.buf.subarray(0, this.size));
        this.buf = newBuf;
        this.reserved = newBuf.length;
    };
    /**
    * Reject byte-level access while the stream sits mid-byte (`bitPos != 0`).
    *
    * libeot returns `EOT_OFF_BYTE_BOUNDARY` for any byte read/write/seek issued
    * before a partial byte has been consumed. The bit-level reader
    * ({@link readNBits}) is the sole legitimate mid-byte accessor and bypasses
    * this guard by touching `buf`/`pos` directly. On valid input the only
    * `readNBits` caller consumes whole bytes per point, so the stream is always
    * byte-aligned when a byte accessor runs and this guard never fires.
    */
    Stream.prototype.ensureByteAligned = function () {
        if (this.bitPos !== 0)
            throw new EotError("OFF_BYTE_BOUNDARY", "Stream: byte-level access at a non-byte boundary (bitPos=".concat(this.bitPos, ", pos=").concat(this.pos, ")"));
    };
    Stream.prototype.ensureWrite = function (n) {
        this.ensureByteAligned();
        var needed = this.pos + n;
        if (needed > this.reserved)
            this.reserve(Math.max(needed, this.reserved * 2 || 256));
        if (needed > this.size)
            this.size = needed;
    };
    Stream.prototype.ensureRead = function (n) {
        this.ensureByteAligned();
        if (this.pos + n > this.size)
            throw new EotError("INSUFFICIENT_BYTES", "Stream: not enough data (need ".concat(n, " bytes at pos ").concat(this.pos, ", size ").concat(this.size, ")"));
    };
    Stream.prototype.seekAbsolute = function (pos) {
        this.ensureByteAligned();
        if (!Number.isInteger(pos) || pos < 0)
            throw new EotError("CORRUPT_FILE", "Stream: invalid seek position (".concat(pos, ")"));
        if (pos > this.size)
            throw new EotError("SEEK_PAST_EOS", "Stream: seek past end (".concat(pos, " > ").concat(this.size, ")"));
        this.pos = pos;
    };
    Stream.prototype.seekRelative = function (offset) {
        this.ensureByteAligned();
        var newPos = this.pos + offset;
        if (!Number.isInteger(offset) || !Number.isInteger(newPos))
            throw new EotError("CORRUPT_FILE", "Stream: invalid relative seek (".concat(offset, ")"));
        if (newPos < 0)
            throw new EotError("CORRUPT_FILE", "Stream: negative seek");
        if (newPos > this.size)
            throw new EotError("SEEK_PAST_EOS", "Stream: seek past end");
        this.pos = newPos;
    };
    Stream.prototype.seekAbsoluteThroughReserve = function (pos) {
        this.ensureByteAligned();
        if (pos > this.reserved)
            throw new EotError("SEEK_PAST_EOS", "Stream: seek to ".concat(pos, " past reserved end (").concat(this.reserved, ")"));
        if (pos > this.size)
            this.size = pos;
        this.pos = pos;
    };
    Stream.prototype.seekRelativeThroughReserve = function (offset) {
        this.seekAbsoluteThroughReserve(this.pos + offset);
    };
    Stream.prototype.readU8 = function () {
        this.ensureRead(1);
        return this.buf[this.pos++];
    };
    Stream.prototype.peekU8 = function () {
        this.ensureRead(1);
        return this.buf[this.pos];
    };
    Stream.prototype.readU16 = function () {
        this.ensureRead(2);
        var v = this.buf[this.pos] << 8 | this.buf[this.pos + 1];
        this.pos += 2;
        return v;
    };
    Stream.prototype.readU24 = function () {
        this.ensureRead(3);
        var v = this.buf[this.pos] << 16 | this.buf[this.pos + 1] << 8 | this.buf[this.pos + 2];
        this.pos += 3;
        return v;
    };
    Stream.prototype.readU32 = function () {
        this.ensureRead(4);
        var v = (this.buf[this.pos] << 24 | this.buf[this.pos + 1] << 16 | this.buf[this.pos + 2] << 8 | this.buf[this.pos + 3]) >>> 0;
        this.pos += 4;
        return v;
    };
    Stream.prototype.readS16 = function () {
        var v = this.readU16();
        return v >= 32768 ? v - 65536 : v;
    };
    Stream.prototype.readS8 = function () {
        var v = this.readU8();
        return v >= 128 ? v - 256 : v;
    };
    Stream.prototype.readChar = function () {
        return String.fromCharCode(this.readU8());
    };
    Stream.prototype.writeU8 = function (v) {
        this.ensureWrite(1);
        this.buf[this.pos++] = v & 255;
    };
    Stream.prototype.writeU16 = function (v) {
        this.ensureWrite(2);
        this.buf[this.pos++] = v >> 8 & 255;
        this.buf[this.pos++] = v & 255;
    };
    Stream.prototype.writeU24 = function (v) {
        if (v < 0 || v > 16777215)
            throw new Error("Stream: writeU24 value out of range: ".concat(v));
        this.ensureWrite(3);
        this.buf[this.pos++] = v >> 16 & 255;
        this.buf[this.pos++] = v >> 8 & 255;
        this.buf[this.pos++] = v & 255;
    };
    Stream.prototype.writeU32 = function (v) {
        this.ensureWrite(4);
        this.buf[this.pos++] = v >>> 24 & 255;
        this.buf[this.pos++] = v >> 16 & 255;
        this.buf[this.pos++] = v >> 8 & 255;
        this.buf[this.pos++] = v & 255;
    };
    Stream.prototype.writeS16 = function (v) {
        this.writeU16(v < 0 ? v + 65536 : v);
    };
    Stream.prototype.writeS8 = function (v) {
        this.writeU8(v < 0 ? v + 256 : v);
    };
    Stream.prototype.readNBits = function (n) {
        if (n === 0)
            return 0;
        if (n > 32)
            throw new Error("Stream: readNBits width out of range: ".concat(n, " (max 32)"));
        var value = 0;
        var bitsRemaining = n;
        while (bitsRemaining > 0) {
            if (this.pos >= this.size && this.bitPos === 0)
                throw new EotError("INSUFFICIENT_BYTES", "Stream: not enough data for bit read");
            var bitsAvailableInByte = 8 - this.bitPos;
            var bitsToRead = Math.min(bitsRemaining, bitsAvailableInByte);
            var shift = bitsAvailableInByte - bitsToRead;
            var mask = (1 << bitsToRead) - 1 << shift;
            value = value << bitsToRead | (this.buf[this.pos] & mask) >> shift;
            this.bitPos += bitsToRead;
            if (this.bitPos >= 8) {
                this.bitPos = 0;
                this.pos++;
            }
            bitsRemaining -= bitsToRead;
        }
        return value >>> 0;
    };
    /**
    * Copy `length` bytes from this stream to `dest`.
    *
    * Both streams must be byte-aligned. The destination must already have the
    * capacity reserved: libeot returns `EOT_OUT_OF_RESERVED_SPACE` when a copy
    * would overrun the reserved buffer, so we throw rather than auto-growing —
    * an under-reservation is a bug we want surfaced, not silently patched.
    */
    Stream.prototype.copyTo = function (dest, length) {
        this.ensureByteAligned();
        dest.ensureByteAligned();
        if (this.pos + length > this.size)
            throw new EotError("INSUFFICIENT_BYTES", "Stream: not enough data for copy");
        var needed = dest.pos + length;
        if (needed > dest.reserved)
            throw new EotError("OUT_OF_RESERVED_SPACE", "Stream: copy of ".concat(length, " bytes exceeds reserved capacity (need ").concat(needed, ", reserved ").concat(dest.reserved, ")"));
        dest.buf.set(this.buf.subarray(this.pos, this.pos + length), dest.pos);
        this.pos += length;
        dest.pos += length;
        if (dest.pos > dest.size)
            dest.size = dest.pos;
    };
    /** Read rest of data as 4-byte-aligned U32 values. Returns 0 on incomplete read. */
    Stream.prototype.readRestAsU32 = function () {
        if (this.pos + 4 > this.size) {
            if (this.pos >= this.size)
                return null;
            var val = 0;
            var remaining = this.size - this.pos;
            for (var i = 0; i < 4; i++) {
                val <<= 8;
                if (i < remaining)
                    val |= this.buf[this.pos + i];
            }
            this.pos = this.size;
            return val >>> 0;
        }
        return this.readU32();
    };
    /**
    * Compute the SFNT-style checksum of bytes in `[beginPos, endPos)` as a sum
    * of big-endian U32 words, zero-padding a final partial word. Bounds strictly
    * on `endPos` (not the stream's `size`), so an unaligned range never folds in
    * bytes past `endPos`. Leaves the stream position unchanged.
    */
    Stream.prototype.checksumU32 = function (beginPos, endPos) {
        if (beginPos > endPos)
            throw new Error("Stream: checksumU32 beginPos ".concat(beginPos, " > endPos ").concat(endPos));
        if (endPos > this.size)
            throw new Error("Stream: checksumU32 endPos ".concat(endPos, " exceeds size ").concat(this.size));
        var sum = 0;
        for (var p = beginPos; p < endPos; p += 4) {
            var word = 0;
            for (var i = 0; i < 4; i++)
                word = word << 8 | (p + i < endPos ? this.buf[p + i] : 0);
            sum = sum + (word >>> 0) >>> 0;
        }
        return sum;
    };
    /** Get a copy of the written data. */
    Stream.prototype.toUint8Array = function () {
        return this.buf.slice(0, this.size);
    };
    return Stream;
}());
//#endregion
//#region src/triplet-encodings.ts
var TRIPLET_ENCODINGS = [
    {
        byteCount: 2,
        xBits: 0,
        yBits: 8,
        deltaX: 0,
        deltaY: 0,
        xSign: 0,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 0,
        yBits: 8,
        deltaX: 0,
        deltaY: 0,
        xSign: 0,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 0,
        yBits: 8,
        deltaX: 0,
        deltaY: 256,
        xSign: 0,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 0,
        yBits: 8,
        deltaX: 0,
        deltaY: 256,
        xSign: 0,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 0,
        yBits: 8,
        deltaX: 0,
        deltaY: 512,
        xSign: 0,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 0,
        yBits: 8,
        deltaX: 0,
        deltaY: 512,
        xSign: 0,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 0,
        yBits: 8,
        deltaX: 0,
        deltaY: 768,
        xSign: 0,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 0,
        yBits: 8,
        deltaX: 0,
        deltaY: 768,
        xSign: 0,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 0,
        yBits: 8,
        deltaX: 0,
        deltaY: 1024,
        xSign: 0,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 0,
        yBits: 8,
        deltaX: 0,
        deltaY: 1024,
        xSign: 0,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 8,
        yBits: 0,
        deltaX: 0,
        deltaY: 0,
        xSign: -1,
        ySign: 0
    },
    {
        byteCount: 2,
        xBits: 8,
        yBits: 0,
        deltaX: 0,
        deltaY: 0,
        xSign: 1,
        ySign: 0
    },
    {
        byteCount: 2,
        xBits: 8,
        yBits: 0,
        deltaX: 256,
        deltaY: 0,
        xSign: -1,
        ySign: 0
    },
    {
        byteCount: 2,
        xBits: 8,
        yBits: 0,
        deltaX: 256,
        deltaY: 0,
        xSign: 1,
        ySign: 0
    },
    {
        byteCount: 2,
        xBits: 8,
        yBits: 0,
        deltaX: 512,
        deltaY: 0,
        xSign: -1,
        ySign: 0
    },
    {
        byteCount: 2,
        xBits: 8,
        yBits: 0,
        deltaX: 512,
        deltaY: 0,
        xSign: 1,
        ySign: 0
    },
    {
        byteCount: 2,
        xBits: 8,
        yBits: 0,
        deltaX: 768,
        deltaY: 0,
        xSign: -1,
        ySign: 0
    },
    {
        byteCount: 2,
        xBits: 8,
        yBits: 0,
        deltaX: 768,
        deltaY: 0,
        xSign: 1,
        ySign: 0
    },
    {
        byteCount: 2,
        xBits: 8,
        yBits: 0,
        deltaX: 1024,
        deltaY: 0,
        xSign: -1,
        ySign: 0
    },
    {
        byteCount: 2,
        xBits: 8,
        yBits: 0,
        deltaX: 1024,
        deltaY: 0,
        xSign: 1,
        ySign: 0
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 1,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 1,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 1,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 1,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 17,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 17,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 17,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 17,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 33,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 33,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 33,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 33,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 49,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 49,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 49,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 1,
        deltaY: 49,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 1,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 1,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 1,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 1,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 17,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 17,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 17,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 17,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 33,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 33,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 33,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 33,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 49,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 49,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 49,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 17,
        deltaY: 49,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 1,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 1,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 1,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 1,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 17,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 17,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 17,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 17,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 33,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 33,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 33,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 33,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 49,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 49,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 49,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 33,
        deltaY: 49,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 1,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 1,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 1,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 1,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 17,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 17,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 17,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 17,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 33,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 33,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 33,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 33,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 49,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 49,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 49,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 2,
        xBits: 4,
        yBits: 4,
        deltaX: 49,
        deltaY: 49,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 1,
        deltaY: 1,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 1,
        deltaY: 1,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 1,
        deltaY: 1,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 1,
        deltaY: 1,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 1,
        deltaY: 257,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 1,
        deltaY: 257,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 1,
        deltaY: 257,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 1,
        deltaY: 257,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 1,
        deltaY: 513,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 1,
        deltaY: 513,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 1,
        deltaY: 513,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 1,
        deltaY: 513,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 257,
        deltaY: 1,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 257,
        deltaY: 1,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 257,
        deltaY: 1,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 257,
        deltaY: 1,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 257,
        deltaY: 257,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 257,
        deltaY: 257,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 257,
        deltaY: 257,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 257,
        deltaY: 257,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 257,
        deltaY: 513,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 257,
        deltaY: 513,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 257,
        deltaY: 513,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 257,
        deltaY: 513,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 513,
        deltaY: 1,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 513,
        deltaY: 1,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 513,
        deltaY: 1,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 513,
        deltaY: 1,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 513,
        deltaY: 257,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 513,
        deltaY: 257,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 513,
        deltaY: 257,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 513,
        deltaY: 257,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 513,
        deltaY: 513,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 513,
        deltaY: 513,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 513,
        deltaY: 513,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 3,
        xBits: 8,
        yBits: 8,
        deltaX: 513,
        deltaY: 513,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 4,
        xBits: 12,
        yBits: 12,
        deltaX: 0,
        deltaY: 0,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 4,
        xBits: 12,
        yBits: 12,
        deltaX: 0,
        deltaY: 0,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 4,
        xBits: 12,
        yBits: 12,
        deltaX: 0,
        deltaY: 0,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 4,
        xBits: 12,
        yBits: 12,
        deltaX: 0,
        deltaY: 0,
        xSign: 1,
        ySign: 1
    },
    {
        byteCount: 5,
        xBits: 16,
        yBits: 16,
        deltaX: 0,
        deltaY: 0,
        xSign: -1,
        ySign: -1
    },
    {
        byteCount: 5,
        xBits: 16,
        yBits: 16,
        deltaX: 0,
        deltaY: 0,
        xSign: 1,
        ySign: -1
    },
    {
        byteCount: 5,
        xBits: 16,
        yBits: 16,
        deltaX: 0,
        deltaY: 0,
        xSign: -1,
        ySign: 1
    },
    {
        byteCount: 5,
        xBits: 16,
        yBits: 16,
        deltaX: 0,
        deltaY: 0,
        xSign: 1,
        ySign: 1
    }
];
//#endregion
//#region src/magnitude.ts
/**
* Read one MTX magnitude-dependent signed value (section 5.3).
* Bits are consumed least-significant-bit first within each byte. Zero is a
* single zero bit; nonzero values are unary magnitude, terminator, then sign.
*/
function readMagnitude(stream) {
    var readBit = function () {
        if (stream.pos >= stream.size)
            throw new EotError("INSUFFICIENT_BYTES", "truncated magnitude value");
        var bit = stream.buf[stream.pos] >> stream.bitPos & 1;
        stream.bitPos++;
        if (stream.bitPos === 8) {
            stream.bitPos = 0;
            stream.pos++;
        }
        return bit;
    };
    if (readBit() === 0)
        return 0;
    var magnitude = 1;
    while (readBit() === 1)
        magnitude++;
    return readBit() === 1 ? -magnitude : magnitude;
}
//#endregion
//#region src/hdmx.ts
/** Translation of the MicroType Express CTF `hdmx` table to its SFNT form. */
var MAX_OUTPUT_BYTES = 67108864;
function fail(message) {
    throw new EotError("CORRUPT_FILE", "invalid hdmx table: ".concat(message));
}
function u16(bytes, offset) {
    return bytes[offset] << 8 | bytes[offset + 1];
}
function u32(bytes, offset) {
    return (bytes[offset] << 24 | bytes[offset + 1] << 16 | bytes[offset + 2] << 8 | bytes[offset + 3]) >>> 0;
}
function putU16(bytes, offset, value) {
    bytes[offset] = value >>> 8;
    bytes[offset + 1] = value & 255;
}
function putU32(bytes, offset, value) {
    bytes[offset] = value >>> 24;
    bytes[offset + 1] = value >>> 16;
    bytes[offset + 2] = value >>> 8;
    bytes[offset + 3] = value;
}
/**
* Decode an `hdmx` table from CTF bytes. Tables stored uncompressed by the
* MTX encoder (complemented version field) are copied with their version
* restored and do not need metric metadata.
*/
function decodeHdmx(data, metadata) {
    if (data.length < 8)
        throw new EotError("INSUFFICIENT_BYTES", "truncated hdmx header");
    var encodedVersion = u16(data, 0);
    var numRecords = u16(data, 2);
    var recordSize = u32(data, 4);
    if (encodedVersion === 65535) {
        if (recordSize < 2 || recordSize % 4 !== 0)
            fail("invalid raw record size");
        var rawLength = 8 + numRecords * recordSize;
        if (!Number.isSafeInteger(rawLength) || rawLength > data.length)
            throw new EotError("INSUFFICIENT_BYTES", "truncated raw hdmx records");
        var raw = data.slice();
        putU16(raw, 0, 0);
        return raw;
    }
    if (encodedVersion !== 0)
        fail("unsupported version ".concat(encodedVersion));
    if (!metadata)
        fail("missing head/hhea/hmtx/maxp metadata");
    var numGlyphs = metadata.numGlyphs, unitsPerEm = metadata.unitsPerEm, numberOfHMetrics = metadata.numberOfHMetrics, hmtx = metadata.hmtx;
    if (!Number.isInteger(numGlyphs) || numGlyphs < 0 || numGlyphs > 65535)
        fail("invalid numGlyphs");
    if (!Number.isInteger(unitsPerEm) || unitsPerEm <= 0 || unitsPerEm > 65535)
        fail("invalid unitsPerEm");
    if (!Number.isInteger(numberOfHMetrics) || numberOfHMetrics < 1 || numberOfHMetrics > numGlyphs)
        fail("invalid numberOfHMetrics");
    if (recordSize !== (numGlyphs + 2 + 3 & -4))
        fail("recordSize does not match glyph count");
    var outputLength = 8 + numRecords * recordSize;
    if (!Number.isSafeInteger(outputLength) || outputLength > MAX_OUTPUT_BYTES)
        fail("decoded table is too large");
    var headerLength = 8 + numRecords * 2;
    if (headerLength > data.length)
        throw new EotError("INSUFFICIENT_BYTES", "truncated hdmx record headers");
    if (headerLength + Math.ceil(numRecords * numGlyphs / 8) > data.length)
        throw new EotError("INSUFFICIENT_BYTES", "truncated hdmx magnitude data");
    var hmtxLength = numberOfHMetrics * 4 + (numGlyphs - numberOfHMetrics) * 2;
    if (hmtx.length < hmtxLength)
        fail("hmtx table is too short");
    var advances = new Uint16Array(numGlyphs);
    var lastAdvance = 0;
    for (var glyph = 0; glyph < numGlyphs; glyph++) {
        if (glyph < numberOfHMetrics)
            lastAdvance = u16(hmtx, glyph * 4);
        advances[glyph] = lastAdvance;
    }
    var out = new Uint8Array(outputLength);
    putU16(out, 0, encodedVersion);
    putU16(out, 2, numRecords);
    putU32(out, 4, recordSize);
    var bits = new Stream(data.subarray(headerLength), data.length - headerLength);
    for (var record = 0; record < numRecords; record++) {
        var sourceRecord = 8 + record * 2;
        var targetRecord = 8 + record * recordSize;
        var ppem = data[sourceRecord];
        out[targetRecord] = ppem;
        out[targetRecord + 1] = data[sourceRecord + 1];
        for (var glyph = 0; glyph < numGlyphs; glyph++) {
            var aw = advances[glyph];
            var rounded64 = Math.floor((64 * ppem * aw + unitsPerEm / 2) / unitsPerEm);
            var width = Math.floor((rounded64 + 32) / 64) + readMagnitude(bits);
            if (width < 0 || width > 255)
                fail("decoded width out of range (".concat(width, ")"));
            out[targetRecord + 2 + glyph] = width;
        }
    }
    return out;
}
//#endregion
//#region src/vdmx.ts
/**
* Decode the MTX magnitude-compressed VDMX table back to OpenType form.
*
* `data` is the bounded compressed byte range, beginning at the table's
* version field. Ratio offsets retain their reconstructed OpenType positions;
* compressed groups are read sequentially from offset[0] and expanded into
* those locations. The returned array may be larger than `data`.
*/
function malformed(message) {
    throw new EotError("CORRUPT_FILE", "malformed VDMX table: ".concat(message));
}
function ensureRange(start, length, limit, what) {
    if (!Number.isInteger(start) || !Number.isInteger(length) || start < 0 || length < 0 || start + length > limit)
        malformed("".concat(what, " is outside the ").concat(limit, "-byte table"));
}
function toInt16$1(value, what) {
    if (!Number.isInteger(value) || value < -32768 || value > 32767)
        malformed("".concat(what, " is outside the signed 16-bit range"));
    return value;
}
function validateUncompressed(data) {
    var _b;
    if (data.length < 6)
        malformed("table is shorter than its header");
    var input = new Stream(data, data.length);
    var version = input.readU16();
    if (version !== 0 && version !== 1)
        malformed("unsupported OpenType version ".concat(version));
    var numRecs = input.readU16();
    var numRatios = input.readU16();
    if (numRecs === 0 || numRatios === 0)
        malformed("table must contain at least one group and ratio");
    var headerEnd = 6 + numRatios * 6;
    ensureRange(6, numRatios * 4, data.length, "ratio records");
    input.seekAbsolute(6 + numRatios * 4);
    var offsets = [];
    for (var i = 0; i < numRatios; i++)
        offsets.push(input.readU16());
    var groupStarts = __spreadArray([], __read(new Set(offsets)), false).sort(function (a, b) { return a - b; });
    if (groupStarts.length !== numRecs)
        malformed("ratio offsets identify ".concat(groupStarts.length, " groups, header declares ").concat(numRecs));
    for (var i = 0; i < groupStarts.length; i++) {
        var start = groupStarts[i];
        if (start < headerEnd)
            malformed("group offset overlaps the header");
        ensureRange(start, 4, data.length, "group ".concat(i, " header"));
        input.seekAbsolute(start);
        var recs = input.readU16();
        input.readU8();
        input.readU8();
        if (recs === 0)
            malformed("group ".concat(i, " has no records"));
        var size = 4 + recs * 6;
        var limit = (_b = groupStarts[i + 1]) !== null && _b !== void 0 ? _b : data.length;
        if (start + size > limit)
            malformed("group ".concat(i, " extends past its table boundary"));
        var previousPpem = -1;
        for (var j = 0; j < recs; j++) {
            var ppem = input.readU16();
            input.readS16();
            input.readS16();
            if (ppem <= previousPpem)
                malformed("group ".concat(i, " has unsorted ppem heights"));
            previousPpem = ppem;
        }
    }
}
/**
* Decode a bounded CTF VDMX table. Uncompressed fallback tables are copied
* through with their original version restored, as prescribed by MTX §5.5.
*/
function decodeVdmx(data) {
    var e_1, _b, e_2, _c;
    var _d;
    if (data.length < 2)
        throw new EotError("INSUFFICIENT_BYTES", "VDMX table is shorter than its version field");
    var input = new Stream(data, data.length);
    var version = input.readU16();
    if (version > 1) {
        var originalVersion = 65535 - version;
        if (originalVersion !== 0 && originalVersion !== 1)
            malformed("unsupported version marker ".concat(version));
        var output_1 = data.slice();
        output_1[0] = originalVersion >>> 8;
        output_1[1] = originalVersion & 255;
        validateUncompressed(output_1);
        return output_1;
    }
    ensureRange(0, 6, data.length, "header");
    var numRecs = input.readU16();
    var numRatios = input.readU16();
    if (numRecs === 0 || numRatios === 0)
        malformed("compressed table must contain at least one group and ratio");
    var headerEnd = 6 + numRatios * 6;
    ensureRange(6, numRatios * 4, data.length, "ratio records");
    input.seekAbsolute(6 + numRatios * 4);
    var offsets = [];
    for (var i = 0; i < numRatios; i++)
        offsets.push(input.readU16());
    var groupStarts = __spreadArray([], __read(new Set(offsets)), false).sort(function (a, b) { return a - b; });
    if (groupStarts.length !== numRecs)
        malformed("ratio offsets identify ".concat(groupStarts.length, " groups, header declares ").concat(numRecs));
    if (groupStarts[0] !== offsets[0])
        malformed("the first ratio offset must point to the first compressed group");
    try {
        for (var groupStarts_1 = __values(groupStarts), groupStarts_1_1 = groupStarts_1.next(); !groupStarts_1_1.done; groupStarts_1_1 = groupStarts_1.next()) {
            var offset = groupStarts_1_1.value;
            if (offset < headerEnd)
                malformed("group offset overlaps the header");
        }
    }
    catch (e_1_1) { e_1 = { error: e_1_1 }; }
    finally {
        try {
            if (groupStarts_1_1 && !groupStarts_1_1.done && (_b = groupStarts_1.return)) _b.call(groupStarts_1);
        }
        finally { if (e_1) throw e_1.error; }
    }
    input.seekAbsolute(offsets[0]);
    var output = data.slice();
    for (var groupIndex = 0; groupIndex < numRecs; groupIndex++) {
        var compressedStart = input.pos;
        var recs = input.readU16();
        var yMaxMultiplier = input.readS16();
        var yMinMultiplier = input.readS16();
        if (recs === 0)
            malformed("group ".concat(groupIndex, " has no records"));
        var expandedSize = 4 + recs * 6;
        var groupStart = groupStarts[groupIndex];
        var groupLimit = (_d = groupStarts[groupIndex + 1]) !== null && _d !== void 0 ? _d : data.length;
        if (groupIndex + 1 < numRecs && groupStart + expandedSize > groupLimit)
            malformed("expanded group ".concat(groupIndex, " overlaps the next group"));
        var records = new Stream(null, 0);
        records.reserve(recs * 6);
        var predictedPpem = 8;
        var previousPpem = -1;
        var startSize = 0;
        var endSize = 0;
        for (var i = 0; i < recs; i++) {
            var ppemError = readMagnitude(input);
            var yMaxError = readMagnitude(input);
            var yMinError = readMagnitude(input);
            var ppem = ppemError + predictedPpem;
            if (!Number.isInteger(ppem) || ppem < 0 || ppem > 65535 || ppem <= previousPpem)
                malformed("group ".concat(groupIndex, " has an invalid or unsorted ppem"));
            var predictedYMax = Math.trunc((ppem * yMaxMultiplier + 1024) / 2048);
            var predictedYMin = -Math.trunc((ppem * yMinMultiplier + 1024) / 2048);
            var yMax = toInt16$1(predictedYMax + yMaxError, "group ".concat(groupIndex, " yMax"));
            var yMin = toInt16$1(predictedYMin + yMinError, "group ".concat(groupIndex, " yMin"));
            if (i === 0)
                startSize = ppem;
            if (i === recs - 1)
                endSize = ppem;
            records.writeU16(ppem);
            records.writeS16(yMax);
            records.writeS16(yMin);
            previousPpem = ppem;
            predictedPpem = ppem + 1;
        }
        if (input.bitPos !== 0) {
            input.pos++;
            input.bitPos = 0;
        }
        ensureRange(compressedStart, input.pos - compressedStart, data.length, "compressed group ".concat(groupIndex));
        var outputGroup = new Stream(null, 0);
        outputGroup.reserve(expandedSize);
        outputGroup.writeU16(recs);
        outputGroup.writeU8(startSize & 255);
        outputGroup.writeU8(endSize & 255);
        try {
            for (var _e = (e_2 = void 0, __values(records.toUint8Array())), _f = _e.next(); !_f.done; _f = _e.next()) {
                var byte = _f.value;
                outputGroup.writeU8(byte);
            }
        }
        catch (e_2_1) { e_2 = { error: e_2_1 }; }
        finally {
            try {
                if (_f && !_f.done && (_c = _e.return)) _c.call(_e);
            }
            finally { if (e_2) throw e_2.error; }
        }
        var groupEnd = groupStart + expandedSize;
        if (groupEnd > output.length) {
            var expanded = new Uint8Array(groupEnd);
            expanded.set(output);
            output = expanded;
        }
        output.set(outputGroup.toUint8Array(), groupStart);
    }
    validateUncompressed(output);
    return output;
}
//#endregion
//#region src/ctf-parser.ts
/**
* CTF (Compact TrueType Font) parser — reconstructs a TrueType font from
* three decompressed CTF streams produced by LZCOMP decompression.
*
* Ported from libeot (MPL 2.0) — parseCTF.c
*
* @see http://www.w3.org/Submission/MTX/
*/
var FLG_ON_CURVE = 1;
var FLG_X_SHORT = 2;
var FLG_Y_SHORT = 4;
var FLG_X_SAME = 16;
var FLG_Y_SAME = 32;
var NPUSHB = 64;
var NPUSHW = 65;
var PUSHB = 176;
var PUSHW = 184;
var ARG_1_AND_2_ARE_WORDS = 1;
var HAVE_SCALE = 8;
var MORE_COMPONENTS = 32;
var HAVE_XY_SCALE = 64;
var HAVE_2_BY_2 = 128;
var HAVE_INSTRUCTIONS = 256;
var INT16_MIN = -32768;
var INT16_MAX = 32767;
/** Wrap a value into signed 16-bit range. */
function toInt16(v) {
    v &= 65535;
    return v >= 32768 ? v - 65536 : v;
}
/**
* Read a 255UShort value from the stream.
*
* Encoding:
*  - 253: next two bytes form a big-endian U16
*  - 255: 253 + next U8
*  - 254: 506 + next U8
*  - else: literal value
*/
function read255UShort(s) {
    var code = s.readU8();
    if (code === 253)
        return s.readU16();
    if (code === 255)
        return 253 + s.readU8();
    if (code === 254)
        return 506 + s.readU8();
    return code;
}
/**
* Read a 255Short value from the stream.
*
* Encoding:
*  - 253: next two bytes form a big-endian S16
*  - 250: negate sign, then re-read next code
*  - 255: 250 + next U8
*  - 254: 500 + next U8
*  - else: literal value
*
* The final result is multiplied by the accumulated sign.
*/
function read255Short(s) {
    var sign = 1;
    var code = s.readU8();
    if (code === 253)
        return s.readS16();
    if (code === 250) {
        sign = -1;
        code = s.readU8();
    }
    var value;
    if (code === 255)
        value = 250 + s.readU8();
    else if (code === 254)
        value = 500 + s.readU8();
    else
        value = code;
    return value * sign;
}
/**
* Decode a delta-encoded CVT (Control Value Table).
*
* Each entry is a signed delta added to a running accumulator and written
* as a big-endian S16 into the output buffer.
*/
function unpackCVT(table, sIn) {
    sIn.seekAbsolute(table.offset);
    var numEntries = sIn.readU16();
    var out = new Stream(null, 0);
    out.reserve(numEntries * 2);
    var lastValue = 0;
    for (var i = 0; i < numEntries; i++) {
        var code = sIn.readU8();
        var val = void 0;
        if (code >= 248)
            val = 238 * (code - 247) + sIn.readU8();
        else if (code >= 239)
            val = -(238 * (code - 239) + sIn.readU8());
        else if (code === 238)
            val = sIn.readS16();
        else
            val = code;
        lastValue = toInt16(lastValue + val);
        out.writeS16(lastValue);
    }
    table.buf = out.toUint8Array();
    table.bufSize = table.buf.length;
}
/**
* Decode hop-coded push data from the instruction stream and emit standard
* TrueType PUSH instructions to the output stream.
*
* Hop codes allow compact representation of alternating patterns:
*  - 0xFB (hop3): `A B → A B A C A`   (consumes 3 more from remaining)
*  - 0xFC (hop4): `A B → A B A C A D A` (consumes 5 more from remaining)
*  - default: read 255Short, consumes 1 from remaining
*
* Values are flushed with the appropriate PUSH opcode when the type changes
* (BYTE vs SHORT) or the accumulated count reaches 255.
*/
function decodePushInstructions(sIn, sOut, pushCount) {
    if (pushCount === 0)
        return;
    var decoded = 0;
    var last = 0;
    var secondLast = 0;
    var remaining = pushCount;
    var isShort = false;
    var runValues = [];
    /** Flush accumulated run values with the appropriate PUSH opcode. */
    function flush() {
        var e_3, _b, e_4, _c;
        if (runValues.length === 0)
            return;
        var count = runValues.length;
        if (isShort) {
            if (count < 8)
                sOut.writeU8(PUSHW + (count - 1));
            else {
                sOut.writeU8(NPUSHW);
                sOut.writeU8(count);
            }
            try {
                for (var runValues_1 = __values(runValues), runValues_1_1 = runValues_1.next(); !runValues_1_1.done; runValues_1_1 = runValues_1.next()) {
                    var v = runValues_1_1.value;
                    sOut.writeS16(v);
                }
            }
            catch (e_3_1) { e_3 = { error: e_3_1 }; }
            finally {
                try {
                    if (runValues_1_1 && !runValues_1_1.done && (_b = runValues_1.return)) _b.call(runValues_1);
                }
                finally { if (e_3) throw e_3.error; }
            }
        }
        else {
            if (count < 8)
                sOut.writeU8(PUSHB + (count - 1));
            else {
                sOut.writeU8(NPUSHB);
                sOut.writeU8(count);
            }
            try {
                for (var runValues_2 = __values(runValues), runValues_2_1 = runValues_2.next(); !runValues_2_1.done; runValues_2_1 = runValues_2.next()) {
                    var v = runValues_2_1.value;
                    sOut.writeU8(v & 255);
                }
            }
            catch (e_4_1) { e_4 = { error: e_4_1 }; }
            finally {
                try {
                    if (runValues_2_1 && !runValues_2_1.done && (_c = runValues_2.return)) _c.call(runValues_2);
                }
                finally { if (e_4) throw e_4.error; }
            }
        }
        runValues.length = 0;
    }
    /** Add a single value to the run, flushing on type change or overflow. */
    function put(v) {
        secondLast = last;
        last = v;
        decoded++;
        var needsShort = v < 0 || v > 255;
        if (runValues.length > 0 && needsShort !== isShort)
            flush();
        if (runValues.length === 0)
            isShort = needsShort;
        runValues.push(v);
        if (runValues.length >= 255)
            flush();
    }
    while (remaining > 0) {
        var code = sIn.peekU8();
        if (code === 251) {
            if (remaining < 3 || decoded < 2)
                throw new EotError("CORRUPT_HOPCODE_DATA", "corrupt hop3 (0xFB) push data: remaining=".concat(remaining, ", decoded=").concat(decoded));
            sIn.readU8();
            var prev = secondLast;
            put(prev);
            put(read255Short(sIn));
            put(prev);
            remaining -= 3;
        }
        else if (code === 252) {
            if (remaining < 5 || decoded < 2)
                throw new EotError("CORRUPT_HOPCODE_DATA", "corrupt hop4 (0xFC) push data: remaining=".concat(remaining, ", decoded=").concat(decoded));
            sIn.readU8();
            var prev = secondLast;
            put(prev);
            put(read255Short(sIn));
            put(prev);
            put(read255Short(sIn));
            put(prev);
            remaining -= 5;
        }
        else {
            put(read255Short(sIn));
            remaining -= 1;
        }
    }
    flush();
}
/**
* Build the TrueType flag byte for a single point.
*
* @param x         X coordinate delta.
* @param y         Y coordinate delta.
* @param onCurve   Whether the point is on the curve.
* @param firstTime Whether this is the very first point in the glyph (always
*                  write the coordinate even if it is zero).
*/
function makeGlyphFlags(x, y, onCurve, firstTime) {
    var flags = 0;
    if (onCurve)
        flags |= FLG_ON_CURVE;
    if (!firstTime && x === 0)
        flags |= FLG_X_SAME;
    else if (x > -256 && x < 0)
        flags |= FLG_X_SHORT;
    else if (x >= 0 && x < 256)
        flags |= 18;
    if (!firstTime && y === 0)
        flags |= FLG_Y_SAME;
    else if (y > -256 && y < 0)
        flags |= FLG_Y_SHORT;
    else if (y >= 0 && y < 256)
        flags |= 36;
    return flags;
}
/**
* Decode a simple glyph from the three CTF streams and write standard
* TrueType glyph data to `out`.
*
* @param numContours  Number of contours (positive).
* @param streams      The three CTF streams:
*                       [0] = glyph / coordinate data,
*                       [1] = push instruction data,
*                       [2] = hinting code data.
* @param out          Output stream receiving TrueType glyph bytes.
* @param calcBBox     If true, compute the bounding box from point data.
* @param minX/minY/maxX/maxY  Explicit bbox values used when calcBBox=false.
*/
function decodeSimpleGlyph(numContours, streams, out, calcBBox, minX, minY, maxX, maxY) {
    if (numContours === 0)
        return;
    var sGlyph = streams[0];
    out.writeS16(numContours);
    var bboxPos = out.pos;
    if (calcBBox) {
        minX = INT16_MAX;
        minY = INT16_MAX;
        maxX = INT16_MIN;
        maxY = INT16_MIN;
        out.writeS16(0);
        out.writeS16(0);
        out.writeS16(0);
        out.writeS16(0);
    }
    else {
        out.writeS16(minX);
        out.writeS16(minY);
        out.writeS16(maxX);
        out.writeS16(maxY);
    }
    var totalPoints = 0;
    for (var c = 0; c < numContours; c++) {
        if (c === 0)
            totalPoints = 1;
        var pointsInContour = read255UShort(sGlyph);
        totalPoints += pointsInContour;
        out.writeU16(totalPoints - 1);
    }
    if (totalPoints > 65535 || totalPoints > sGlyph.size - sGlyph.pos)
        throw new EotError("CORRUPT_FILE", "glyph declares implausible point count ".concat(totalPoints));
    var flagBytes = sGlyph.buf.subarray(sGlyph.pos, sGlyph.pos + totalPoints);
    sGlyph.pos += totalPoints;
    var xDeltas = new Int16Array(totalPoints);
    var yDeltas = new Int16Array(totalPoints);
    var cumulativeX = 0;
    var cumulativeY = 0;
    var gBuf = sGlyph.buf;
    var gSize = sGlyph.size;
    var gPos = sGlyph.pos;
    for (var i = 0; i < totalPoints; i++) {
        var enc = TRIPLET_ENCODINGS[flagBytes[i] & 127];
        var n = enc.byteCount - 1;
        if (gPos + n > gSize) {
            sGlyph.pos = gPos;
            throw new EotError("INSUFFICIENT_BYTES", "Stream: not enough data for bit read");
        }
        var raw = 0;
        for (var k = 0; k < n; k++)
            raw = raw << 8 | gBuf[gPos++];
        var yBits = enc.yBits;
        var rawX = raw >>> yBits;
        var rawY = raw & (1 << yBits) - 1;
        var dx = rawX + enc.deltaX;
        var dy = rawY + enc.deltaY;
        if (enc.xSign !== 0)
            dx *= enc.xSign;
        if (enc.ySign !== 0)
            dy *= enc.ySign;
        xDeltas[i] = dx;
        yDeltas[i] = dy;
        cumulativeX += xDeltas[i];
        cumulativeY += yDeltas[i];
        if (calcBBox) {
            if (cumulativeX < minX)
                minX = cumulativeX;
            if (cumulativeX > maxX)
                maxX = cumulativeX;
            if (cumulativeY < minY)
                minY = cumulativeY;
            if (cumulativeY > maxY)
                maxY = cumulativeY;
        }
    }
    sGlyph.pos = gPos;
    var codeSizeLocation = out.pos;
    out.writeU16(0);
    var pushCount = read255UShort(sGlyph);
    decodePushInstructions(streams[1], out, pushCount);
    var codeSize = read255UShort(sGlyph);
    if (codeSize > 0) {
        out.reserve(out.pos + codeSize);
        streams[2].copyTo(out, codeSize);
    }
    var unpackedCodeSize = out.pos - (codeSizeLocation + 2);
    var savedPos = out.pos;
    out.seekAbsolute(codeSizeLocation);
    out.writeU16(unpackedCodeSize);
    out.seekAbsolute(savedPos);
    out.reserve(out.pos + 5 * totalPoints);
    var oBuf = out.buf;
    var o = out.pos;
    for (var i = 0; i < totalPoints; i++) {
        var onCurve = (flagBytes[i] & 128) === 0;
        oBuf[o++] = makeGlyphFlags(xDeltas[i], yDeltas[i], onCurve, i === 0);
    }
    for (var i = 0; i < totalPoints; i++) {
        var x = xDeltas[i];
        if (i === 0 || x !== 0) {
            if (x > -256 && x < 256)
                oBuf[o++] = x < 0 ? -x : x;
            else {
                oBuf[o++] = x >> 8 & 255;
                oBuf[o++] = x & 255;
            }
        }
    }
    for (var i = 0; i < totalPoints; i++) {
        var y = yDeltas[i];
        if (i === 0 || y !== 0) {
            if (y > -256 && y < 256)
                oBuf[o++] = y < 0 ? -y : y;
            else {
                oBuf[o++] = y >> 8 & 255;
                oBuf[o++] = y & 255;
            }
        }
    }
    out.pos = o;
    if (o > out.size)
        out.size = o;
    if (calcBBox) {
        var endPos = out.pos;
        out.seekAbsolute(bboxPos);
        out.writeS16(minX);
        out.writeS16(minY);
        out.writeS16(maxX);
        out.writeS16(maxY);
        out.seekAbsolute(endPos);
    }
}
/**
* Decode a composite glyph from the CTF streams.
*
* Composite glyphs reference other glyphs via component records. Each
* component specifies a glyph index, positioning arguments, and optional
* transformation matrices.
*/
function decodeCompositeGlyph(streams, out) {
    var sGlyph = streams[0];
    out.writeS16(-1);
    out.writeS16(sGlyph.readS16());
    out.writeS16(sGlyph.readS16());
    out.writeS16(sGlyph.readS16());
    out.writeS16(sGlyph.readS16());
    var flags = 0;
    do {
        flags = sGlyph.readU16();
        var glyphIndex = sGlyph.readU16();
        out.writeU16(flags);
        out.writeU16(glyphIndex);
        var argBytes = void 0;
        if (flags & ARG_1_AND_2_ARE_WORDS)
            argBytes = 4;
        else
            argBytes = 2;
        out.reserve(out.pos + argBytes + 8);
        sGlyph.copyTo(out, argBytes);
        var transformBytes = 0;
        if (flags & HAVE_2_BY_2)
            transformBytes = 8;
        else if (flags & HAVE_XY_SCALE)
            transformBytes = 4;
        else if (flags & HAVE_SCALE)
            transformBytes = 2;
        if (transformBytes > 0)
            sGlyph.copyTo(out, transformBytes);
    } while (flags & MORE_COMPONENTS);
    if (flags & HAVE_INSTRUCTIONS) {
        var numInstrPos = out.pos;
        out.writeU16(0);
        var pushCount = read255UShort(sGlyph);
        decodePushInstructions(streams[1], out, pushCount);
        var codeSize = read255UShort(sGlyph);
        if (codeSize > 0) {
            out.reserve(out.pos + codeSize);
            streams[2].copyTo(out, codeSize);
        }
        var numInstr = out.pos - (numInstrPos + 2);
        var savedPos = out.pos;
        out.seekAbsolute(numInstrPos);
        out.writeU16(numInstr);
        out.seekAbsolute(savedPos);
    }
}
/**
* Decode a single glyph from the CTF streams.
*
* The first value read is `numContours`:
*  - Negative: composite glyph
*  - 0x7FFF:  simple glyph with explicit bbox and actual contour count
*  - Other:   simple glyph with computed bbox
*/
function decodeGlyph(streams, out) {
    var numContours = streams[0].readS16();
    if (numContours < 0)
        decodeCompositeGlyph(streams, out);
    else if (numContours === 32767)
        decodeSimpleGlyph(streams[0].readS16(), streams, out, false, streams[0].readS16(), streams[0].readS16(), streams[0].readS16(), streams[0].readS16());
    else
        decodeSimpleGlyph(numContours, streams, out, true, 0, 0, 0, 0);
}
/**
* Iterate over every glyph, decode it, and build the `glyf` and `loca`
* tables.
*
* @param glyf      The glyf table descriptor (output buf/bufSize are set).
* @param loca      The loca table descriptor (output buf/bufSize are set).
* @param headData  Parsed `head` table fields.
* @param maxpData  Parsed `maxp` table fields.
* @param streams   The three CTF streams.
*/
function populateGlyfAndLoca(glyf, loca, headData, maxpData, streams) {
    var e_5, _b;
    var numGlyphs = maxpData.numGlyphs;
    streams[0].seekAbsolute(glyf.offset);
    streams[1].seekAbsolute(0);
    streams[2].seekAbsolute(0);
    var maxGlyphSize = 10 + 2 * maxpData.maxContours + 2 + maxpData.maxSizeOfInstructions + 256 + 5 * maxpData.maxPoints + 4 * maxpData.maxComponentElements * 6 + 256;
    var outStream = new Stream(null, 0);
    outStream.reserve(Math.min(numGlyphs * 256, 2 * (streams[0].size - streams[0].pos) + 4 * streams[1].size + streams[2].size + 2 * numGlyphs + 256));
    var locaOffsets = [0];
    for (var i = 0; i < numGlyphs; i++) {
        outStream.pos;
        outStream.reserve(outStream.pos + maxGlyphSize);
        decodeGlyph(streams, outStream);
        if (outStream.pos & 1)
            outStream.writeU8(0);
        locaOffsets.push(outStream.pos);
    }
    var useLongLoca = headData.indexToLocFormat === 1 || outStream.pos > 131070;
    if (useLongLoca && headData.indexToLocFormat === 0) {
        headData.table.buf[50] = 0;
        headData.table.buf[51] = 1;
        headData.indexToLocFormat = 1;
    }
    var locaStream = new Stream(null, 0);
    locaStream.reserve((numGlyphs + 1) * (useLongLoca ? 4 : 2));
    try {
        for (var locaOffsets_1 = __values(locaOffsets), locaOffsets_1_1 = locaOffsets_1.next(); !locaOffsets_1_1.done; locaOffsets_1_1 = locaOffsets_1.next()) {
            var offset = locaOffsets_1_1.value;
            if (useLongLoca)
                locaStream.writeU32(offset);
            else
                locaStream.writeU16(offset / 2);
        }
    }
    catch (e_5_1) { e_5 = { error: e_5_1 }; }
    finally {
        try {
            if (locaOffsets_1_1 && !locaOffsets_1_1.done && (_b = locaOffsets_1.return)) _b.call(locaOffsets_1);
        }
        finally { if (e_5) throw e_5.error; }
    }
    glyf.buf = outStream.buf.subarray(0, outStream.size);
    glyf.bufSize = glyf.buf.length;
    loca.buf = locaStream.buf.subarray(0, locaStream.size);
    loca.bufSize = loca.buf.length;
}
/**
* Parse the `head` table to extract `indexToLocFormat`.
*
* `indexToLocFormat` is at byte offset 50 within the `head` table.
*/
function parseHead(table, hasGlyf) {
    if (!hasGlyf)
        return {
            indexToLocFormat: 0,
            table: table
        };
    if (table.bufSize < 54 || table.buf.length < 54)
        throw new EotError("MALFORMED_HEAD_TABLE", "head table too small: ".concat(table.bufSize, " bytes (need 54)"));
    var s = new Stream(table.buf, table.bufSize);
    s.seekAbsolute(50);
    var indexToLocFormat = s.readS16();
    if (indexToLocFormat !== 0 && indexToLocFormat !== 1)
        throw new EotError("MALFORMED_HEAD_TABLE", "invalid head indexToLocFormat: ".concat(indexToLocFormat));
    return {
        indexToLocFormat: indexToLocFormat,
        table: table
    };
}
/**
* Parse the `maxp` table to extract key metrics.
*
* Version 1.0 (0x00010000) layout:
*   offset 0  : version          U32
*   offset 4  : numGlyphs        U16
*   offset 6  : maxPoints         U16
*   offset 8  : maxContours       U16
*   offset 10 : maxCompositePoints U16
*   offset 12 : maxCompositeContours U16
*   offset 14 : maxZones          U16
*   offset 16 : maxTwilightPoints U16
*   offset 18 : maxStorage        U16
*   offset 20 : maxFunctionDefs   U16
*   offset 22 : maxInstructionDefs U16
*   offset 24 : maxStackElements  U16
*   offset 26 : maxSizeOfInstructions U16
*   offset 28 : maxComponentElements  U16
*   ... (remaining fields not needed)
*/
function parseMaxp(table) {
    var s = new Stream(table.buf, table.bufSize);
    var version = s.readU32();
    var numGlyphs = s.readU16();
    var maxPoints = 0;
    var maxContours = 0;
    var maxSizeOfInstructions = 0;
    var maxComponentElements = 0;
    if (version === 65536) {
        maxPoints = s.readU16();
        maxContours = s.readU16();
        s.readU16();
        s.readU16();
        s.readU16();
        s.readU16();
        s.readU16();
        s.readU16();
        s.readU16();
        s.readU16();
        maxSizeOfInstructions = s.readU16();
        maxComponentElements = s.readU16();
    }
    return {
        numGlyphs: numGlyphs,
        maxPoints: maxPoints,
        maxContours: maxContours,
        maxSizeOfInstructions: maxSizeOfInstructions,
        maxComponentElements: maxComponentElements
    };
}
var SFNT_TRUETYPE = 65536;
var SFNT_OTTO = 1330926671;
var SFNT_TRUE = 1953658213;
/**
* The sfnt version for the rebuilt font. It follows the outlines actually
* present rather than trusting the header alone: a font whose outlines are a
* CFF table must say 'OTTO', or strict rasterisers (Apple CoreText) reject
* it, and one with `glyf` outlines must not say 'OTTO'.
*/
function resolveScalarType(header, cffOutlines) {
    if (cffOutlines)
        return SFNT_OTTO;
    return header === SFNT_TRUE ? SFNT_TRUE : SFNT_TRUETYPE;
}
/**
* Parse a CTF (Compact TrueType Font) container from the three LZCOMP-
* decompressed streams and produce an `SFNTContainer` that can be
* serialized into a standard TrueType font file.
*
* @param streams  Three `Stream` objects produced by MTX decompression:
*                   [0] = glyph / table data
*                   [1] = push instruction data
*                   [2] = hinting code data
* @returns An `SFNTContainer` holding all reconstructed SFNT tables.
*/
function parseCTF(streams, options) {
    var e_6, _b, e_7, _c;
    var s0 = streams[0];
    var headerScalarType = s0.readU32();
    var numTables = s0.readU16();
    s0.readU16();
    s0.readU16();
    s0.readU16();
    var tables = [];
    var glyfIdx = -1;
    var locaIdx = -1;
    var maxpIdx = -1;
    var headIdx = -1;
    var hmtxIdx = -1;
    for (var i = 0; i < numTables; i++) {
        var tag = s0.readChar() + s0.readChar() + s0.readChar() + s0.readChar();
        s0.seekRelative(4);
        var table = {
            tag: tag,
            offset: s0.readU32(),
            bufSize: s0.readU32(),
            buf: /* @__PURE__ */ new Uint8Array(0),
            checksum: 0
        };
        var idx = tables.length;
        tables.push(table);
        if (tag === "glyf")
            glyfIdx = idx;
        else if (tag === "loca")
            locaIdx = idx;
        else if (tag === "maxp")
            maxpIdx = idx;
        else if (tag === "head")
            headIdx = idx;
        else if (tag === "hmtx")
            hmtxIdx = idx;
        else if (tag === "cvt ") { }
    }
    for (var i = 0; i < tables.length; i++) {
        var table = tables[i];
        if (table.tag === "glyf" || table.tag === "loca")
            continue;
        if (table.tag === "cvt ") {
            unpackCVT(table, s0);
            continue;
        }
        s0.seekAbsolute(table.offset);
        if (table.bufSize > s0.size - table.offset)
            throw new EotError("INSUFFICIENT_BYTES", "truncated ".concat(table.tag, " table"));
        table.buf = s0.buf.slice(table.offset, table.offset + table.bufSize);
        if (table.tag === "head") {
            if (table.bufSize < 12)
                throw new EotError("MALFORMED_HEAD_TABLE", "head table too small: ".concat(table.bufSize, " bytes (need at least 12)"));
            table.buf[8] = 0;
            table.buf[9] = 0;
            table.buf[10] = 0;
            table.buf[11] = 0;
        }
    }
    if (maxpIdx < 0)
        throw new EotError("NO_MAXP_TABLE", "CTF font is missing a maxp table");
    if (headIdx < 0)
        throw new EotError("NO_HEAD_TABLE", "CTF font is missing a head table");
    if (hmtxIdx < 0)
        throw new EotError("NO_HMTX_TABLE", "CTF font is missing an hmtx table");
    var headData = parseHead(tables[headIdx], glyfIdx >= 0);
    var maxpData = parseMaxp(tables[maxpIdx]);
    var maxpVersion = new DataView(tables[maxpIdx].buf.buffer, tables[maxpIdx].buf.byteOffset, tables[maxpIdx].buf.byteLength).getUint32(0);
    var cffOutlines = tables.some(function (t) { return t.tag === "CFF " || t.tag === "CFF2"; }) && glyfIdx < 0;
    if (maxpVersion !== 65536 && !cffOutlines)
        emitWarning(options, "WARN_MAXP_VERSION", "maxp version 0x".concat(maxpVersion.toString(16), " is not 1.0; MTX carries TrueType outlines, glyph limits were not read"));
    var _loop_1 = function (tag) {
        if (!tables.some(function (t) { return t.tag === tag; }))
            emitWarning(options, "WARN_MISSING_TABLE", "CTF font has no ".concat(tag, " table; the reconstructed font may be unusable in some renderers"));
    };
    try {
        for (var _d = __values([
            "hhea",
            "cmap",
            "post",
            "name",
            "OS/2"
        ]), _e = _d.next(); !_e.done; _e = _d.next()) {
            var tag = _e.value;
            _loop_1(tag);
        }
    }
    catch (e_6_1) { e_6 = { error: e_6_1 }; }
    finally {
        try {
            if (_e && !_e.done && (_b = _d.return)) _b.call(_d);
        }
        finally { if (e_6) throw e_6.error; }
    }
    try {
        for (var tables_1 = __values(tables), tables_1_1 = tables_1.next(); !tables_1_1.done; tables_1_1 = tables_1.next()) {
            var table = tables_1_1.value;
            if (table.tag === "VDMX") {
                table.buf = decodeVdmx(table.buf);
                table.bufSize = table.buf.length;
            }
            else if (table.tag === "hdmx") {
                if ((table.buf.length >= 2 ? table.buf[0] << 8 | table.buf[1] : -1) === 0) {
                    var hhea = tables.find(function (candidate) { return candidate.tag === "hhea"; });
                    if (!hhea || hhea.buf.length < 36)
                        throw new EotError("CORRUPT_FILE", "compressed hdmx requires a complete hhea table");
                    var head = tables[headIdx].buf;
                    table.buf = decodeHdmx(table.buf, {
                        numGlyphs: maxpData.numGlyphs,
                        unitsPerEm: head[18] << 8 | head[19],
                        numberOfHMetrics: hhea.buf[34] << 8 | hhea.buf[35],
                        hmtx: tables[hmtxIdx].buf
                    });
                }
                else
                    table.buf = decodeHdmx(table.buf);
                table.bufSize = table.buf.length;
            }
        }
    }
    catch (e_7_1) { e_7 = { error: e_7_1 }; }
    finally {
        try {
            if (tables_1_1 && !tables_1_1.done && (_c = tables_1.return)) _c.call(tables_1);
        }
        finally { if (e_7) throw e_7.error; }
    }
    if (glyfIdx >= 0) {
        if (locaIdx < 0) {
            var locaTable = {
                tag: "loca",
                offset: 0,
                bufSize: 0,
                buf: /* @__PURE__ */ new Uint8Array(0),
                checksum: 0
            };
            locaIdx = tables.length;
            tables.push(locaTable);
        }
        populateGlyfAndLoca(tables[glyfIdx], tables[locaIdx], headData, maxpData, streams);
    }
    return {
        tables: tables,
        scalarType: resolveScalarType(headerScalarType, cffOutlines)
    };
}
//#endregion
//#region src/ahuff.ts
/**
* Return the number of bits required to represent the non-negative integer `x`.
* Equivalent to floor(log2(x)) + 1 for x > 0.
*
* libeot's `BitsUsed(0)` returns 1 (its mask cascade falls through to the low
* bit); we match that so the function is a faithful port. In practice this
* module only ever calls it with x > 0, so the zero case is defensive.
*/
function bitsUsed(x) {
    if (x <= 0)
        return 1;
    return 32 - Math.clz32(x);
}
var AHuff = (_a = /** @class */ (function () {
        function AHuff(bio, range) {
            this.bio = bio;
            this.range = range;
            this.bitCount = bitsUsed(range - 1);
            this.bitCount2 = 0;
            if (range > 256 && range < 512)
                this.bitCount2 = bitsUsed(range - 256 - 1) + 1;
            var treeSize = 2 * range;
            var up = this.up = new Int32Array(treeSize);
            var child = this.child = new Int32Array(2 * treeSize);
            var code = this.code = new Int32Array(treeSize).fill(-1);
            this.weight = new Int32Array(treeSize);
            for (var i = 2; i < treeSize; i++) {
                up[i] = i >> 1;
                this.weight[i] = 1;
            }
            for (var i = 1; i < range; i++) {
                child[2 * i] = 2 * i;
                child[2 * i + 1] = 2 * i + 1;
            }
            this.symbolIndex = new Int32Array(range);
            for (var i = 0; i < range; i++) {
                var leafIdx = range + i;
                code[leafIdx] = i;
                child[2 * leafIdx] = -1;
                child[2 * leafIdx + 1] = -1;
                this.symbolIndex[i] = leafIdx;
            }
            this.initWeight(_a.ROOT);
            if (this.bitCount2 !== 0) {
                this.updateWeight(this.symbolIndex[256]);
                this.updateWeight(this.symbolIndex[257]);
                var dup2Sym = range - 3;
                for (var i = 0; i < 12; i++)
                    this.updateWeight(this.symbolIndex[dup2Sym]);
                var dup4Sym = range - 2;
                for (var i = 0; i < 6; i++)
                    this.updateWeight(this.symbolIndex[dup4Sym]);
            }
            else
                for (var j = 0; j < 2; j++)
                    for (var i = 0; i < range; i++)
                        this.updateWeight(this.symbolIndex[i]);
        }
        /**
        * Decode one symbol from the bit stream.
        *
        * Starting at ROOT, read one bit at a time:
        *   - 0 → go left
        *   - 1 → go right
        * Continue until a leaf (code >= 0) is reached.  Then update the
        * tree weights and return the symbol code.
        */
        AHuff.prototype.readSymbol = function () {
            var bio = this.bio;
            var code = this.code;
            var child = this.child;
            var a = _a.ROOT;
            var symbol;
            var data = bio.data;
            var size = bio.size;
            var index = bio.index;
            var byte = bio.bitBuffer;
            var count = bio.bitCount;
            do {
                if (count === 0) {
                    if (index >= size) {
                        bio.index = index;
                        bio.bitBuffer = byte;
                        bio.bitCount = 0;
                        throw new EotError("INSUFFICIENT_BYTES", "BitIO: end of data");
                    }
                    byte = data[index++];
                    count = 8;
                }
                a = child[a << 1 | byte >> --count & 1];
                symbol = code[a];
            } while (symbol < 0);
            bio.index = index;
            bio.bitBuffer = byte;
            bio.bitCount = count;
            this.updateWeight(a);
            return symbol;
        };
        /**
        * Increment the weight of node `a` and propagate up to ROOT, swapping
        * nodes as necessary to maintain the sibling property (nodes in
        * non-increasing weight order by index). For each node below ROOT: if its
        * predecessor has the same weight, scan back to the first node with that
        * weight and swap with it (unless that is ROOT), then bump the weight and
        * move to the parent. Finally ROOT's weight is incremented.
        */
        AHuff.prototype.updateWeight = function (a) {
            var weight = this.weight;
            var up = this.up;
            for (; a !== _a.ROOT; a = up[a]) {
                var weightA = weight[a];
                var b = a - 1;
                if (weight[b] === weightA) {
                    do
                        b--;
                    while (weight[b] === weightA);
                    b++;
                    if (b > _a.ROOT) {
                        this.swapNodes(a, b);
                        a = b;
                    }
                }
                weight[a] = weightA + 1;
            }
            weight[_a.ROOT]++;
        };
        /**
        * Swap the content (children, code, weight) of two nodes. The `up`
        * pointer stays with the position, so afterwards the children of each
        * swapped internal node get their parent pointer fixed, and leaves get
        * their `symbolIndex` entry updated.
        */
        AHuff.prototype.swapNodes = function (a, b) {
            var _b = this, child = _b.child, code = _b.code, weight = _b.weight, up = _b.up;
            var a2 = a << 1;
            var b2 = b << 1;
            var t = child[a2];
            child[a2] = child[b2];
            child[b2] = t;
            t = child[a2 + 1];
            child[a2 + 1] = child[b2 + 1];
            child[b2 + 1] = t;
            t = code[a];
            code[a] = code[b];
            code[b] = t;
            t = weight[a];
            weight[a] = weight[b];
            weight[b] = t;
            var c = code[a];
            if (c < 0) {
                up[child[a2]] = a;
                up[child[a2 + 1]] = a;
            }
            else
                this.symbolIndex[c] = a;
            c = code[b];
            if (c < 0) {
                up[child[b2]] = b;
                up[child[b2 + 1]] = b;
            }
            else
                this.symbolIndex[c] = b;
        };
        /**
        * Recursively compute weights for internal nodes after the initial
        * tree construction.  Leaf weights are already set to 1.
        *
        * weight(internal) = weight(left) + weight(right)
        */
        AHuff.prototype.initWeight = function (a) {
            if (this.code[a] >= 0)
                return this.weight[a];
            return this.weight[a] = this.initWeight(this.child[2 * a]) + this.initWeight(this.child[2 * a + 1]);
        };
        return AHuff;
    }()),
    _a.ROOT = 1,
    _a);
//#endregion
//#region src/bitio.ts
/**
* BitIO - MSB-first bit-level reader from a Uint8Array buffer.
*
* Ported from libeot (MPL 2.0) MTX_BITIO implementation.
*
* The reader maintains a single-byte shift register (`bitBuffer`).
* Bits are consumed most-significant-bit first.  When all bits in the
* current byte have been shifted out, the next byte is loaded
* automatically.
*/
var BitIO = /** @class */ (function () {
    /**
    * @param data   Source byte buffer.
    * @param offset Starting byte offset into `data`.
    * @param size   Absolute end index into `data` (exclusive) — reading stops
    *               once `index` reaches it. Defaults to `data.length`. Clamped
    *               to `data.length` so an over-large value cannot read past the
    *               end of the buffer and silently yield zero bits.
    */
    function class_1(data, offset, size) {
        if (offset === void 0) { offset = 0; }
        /** @internal Current byte being consumed. */ this.bitBuffer = 0;
        /** @internal Bits of `bitBuffer` not yet consumed (0..8). */ this.bitCount = 0;
        this.data = data;
        this.index = offset;
        this.size = Math.min(size !== null && size !== void 0 ? size : data.length, data.length);
    }
    /**
    * Read a single bit from the stream.
    *
    * Mirrors `MTX_BITIO_input_bit`:
    *   - If `bitCount` has reached 0, load the next byte into `bitBuffer`
    *     and reset `bitCount` to 7.
    *   - Shift `bitBuffer` left by 1.
    *   - Return whether bit 8 (0x100) is set (i.e. the MSB that was
    *     shifted out of the original byte value).
    */
    class_1.prototype.inputBit = function () {
        return this.nextBit() !== 0;
    };
    /** Same as {@link inputBit} but returns 0 | 1 (hot-path variant). */
    class_1.prototype.nextBit = function () {
        if (this.bitCount === 0) {
            if (this.index >= this.size)
                throw new EotError("INSUFFICIENT_BYTES", "BitIO: end of data");
            this.bitBuffer = this.data[this.index++];
            this.bitCount = 8;
        }
        return this.bitBuffer >> --this.bitCount & 1;
    };
    /**
    * Read an unsigned integer of `numberOfBits` width, MSB first.
    *
    * Mirrors `MTX_BITIO_ReadValue`: accumulates bits from the most
    * significant down to the least significant.
    */
    class_1.prototype.readValue = function (numberOfBits) {
        var value = 0;
        for (var i = numberOfBits - 1; i >= 0; i--)
            value = value << 1 | this.nextBit();
        return value >>> 0;
    };
    return class_1;
}());
//#endregion
//#region src/lzcomp.ts
/**
* LZCOMP decompression module for MicroType Express (MTX) fonts.
* Ported from libeot (MPL 2.0) — lzcomp.c
*
* Implements LZCOMP sliding-window decompression with optional
* run-length decoding on the output stream.
*/
/** Number of bits used for each length chunk (excluding the stop bit). */
var LEN_WIDTH = 3;
/** Number of bits used for each distance chunk. */
var DIST_WIDTH = 3;
/** Number of data bits per length chunk (LEN_WIDTH − 1). */
var BIT_RANGE = 2;
/** Distances >= this threshold add 1 to the decoded length. */
var MAX_2BYTE_DIST = 512;
/**
* Size (in bytes) of the deterministic preload written into the sliding
* window before decompression begins: 2 * 32 * 96 + 4 * 256 = 7168.
*/
var PRELOAD_SIZE = 7168;
/** Minimum match length returned by decodeLength. */
var LEN_MIN = 2;
/** Minimum match distance returned by decodeDistance. */
var DIST_MIN = 1;
/**
* Hard cap on the declared LZCOMP output length. The `out_len` header field is
* 24 bits wide (see `readValue(24)` below), so 2^24 - 1 is the largest value an
* encoder can legally emit; anything above that is corrupt input, not a large
* font. Capping lower would reject legitimate fonts (e.g. large CJK glyph
* streams that exceed a few MiB uncompressed).
*/
var MAX_OUT_LEN = (1 << 24) - 1;
/**
* Hard cap on RLE-expanded output growth. RLE can legally expand up to 255:1
* per escape triple, so the expanded stream can exceed the declared `out_len`.
* Bound it a small multiple above the 24-bit maximum to stop a hostile stream
* from growing the buffer without limit while still admitting real fonts.
*/
var MAX_OUT = 67108864;
/** Message prefix of the error raised when output would exceed the budget. */
var OUTPUT_BUDGET_EXCEEDED = "LZCOMP output exceeds maximum size budget";
var RLE_INITIAL = 0;
var RLE_NORMAL = 1;
var RLE_SEEN_ESCAPE = 2;
var RLE_NEED_BYTE = 3;
/**
* Compute the number of distance ranges and the special-symbol indices
* that depend on the total output length.
*/
function setDistRange(length) {
    var numDistRanges = 1;
    var distMax = DIST_MIN + ((1 << DIST_WIDTH * numDistRanges) - 1);
    while (distMax < length) {
        numDistRanges++;
        if (numDistRanges > 8)
            throw new EotError("MTX_ERROR", "LZCOMP setDistRange: numDistRanges exceeds bound (8)");
        distMax = DIST_MIN + ((1 << DIST_WIDTH * numDistRanges) - 1);
    }
    var DUP2 = 256 + 8 * numDistRanges;
    var DUP4 = DUP2 + 1;
    var DUP6 = DUP4 + 1;
    var NUM_SYMS = DUP6 + 1;
    return {
        numDistRanges: numDistRanges,
        distMax: distMax,
        DUP2: DUP2,
        DUP4: DUP4,
        DUP6: DUP6,
        NUM_SYMS: NUM_SYMS
    };
}
/**
* Pre-load the sliding window with deterministic data so that
* early back-references can match known patterns.
*/
function initializeModel(window) {
    var i = 0;
    for (var k = 0; k < 32; k++)
        for (var j_1 = 0; j_1 < 96; j_1++) {
            window[i++] = k;
            window[i++] = j_1;
        }
    var j = 0;
    while (i < PRELOAD_SIZE && j < 256) {
        window[i++] = j;
        window[i++] = j;
        window[i++] = j;
        window[i++] = j;
        j++;
    }
}
/**
* Decode a match length using a stop-bit scheme.
*
* Each chunk contributes `BIT_RANGE` (2) data bits.  Bit 2 of each chunk
* is the *continuation* bit:
*   - bit 2 SET   (1) → more chunks follow
*   - bit 2 CLEAR (0) → this is the last chunk
*
* The first chunk is derived from the already-decoded main symbol; any
* subsequent chunks are read from `lenEcoder`.
*
* @param lenEcoder   Adaptive Huffman tree for length continuation symbols.
* @param symbol      The main symbol that triggered a copy operation (>= 256).
* @param numDistRangesOut  Output parameter — receives the number of distance
*                          ranges for the subsequent distance decode.
* @returns The decoded match length (>= LEN_MIN).
*/
function decodeLength(lenEcoder, symbol) {
    var mask = 4;
    var firstTime = true;
    var value = 0;
    var done;
    var iters = 0;
    do {
        if (++iters > 16)
            throw new EotError("MTX_ERROR", "LZCOMP decodeLength: iteration cap exceeded");
        var bits = void 0;
        if (firstTime) {
            bits = symbol - 256;
            firstTime = false;
            bits &= 7;
        }
        else
            bits = lenEcoder.readSymbol();
        done = (bits & mask) === 0;
        bits &= -5;
        value <<= BIT_RANGE;
        value |= bits;
    } while (!done);
    value += LEN_MIN;
    return value;
}
/**
* Decode a match distance by reading `numDistRanges` chunks from the
* distance Huffman tree, each contributing `DIST_WIDTH` (3) bits.
*/
function decodeDistance(distEcoder, numDistRanges) {
    var value = 0;
    for (var i = numDistRanges; i > 0; i--) {
        var bits = distEcoder.readSymbol();
        value <<= DIST_WIDTH;
        value |= bits;
    }
    value += DIST_MIN;
    return value;
}
/**
* Decompress an LZCOMP-compressed byte stream, optionally followed by
* run-length decoding.
*
* @param data     The compressed input bytes.
* @param size     Number of valid bytes in `data`.
* @param version  MTX version flag.  When version !== 1 a leading bit
*                 indicates whether run-length encoding is used on the
*                 output.
* @param maxOut   Optional output budget in bytes. Both the declared length
*                 and RLE-expanded output are rejected once they exceed it,
*                 before the corresponding buffers are allocated or grown.
* @returns The decompressed byte array.
*/
function lzcompDecompress(data, size, version, maxOut) {
    if (maxOut === void 0) { maxOut = MAX_OUT; }
    var bio = new BitIO(data, 0, size);
    var outCap = Math.min(MAX_OUT, maxOut);
    var usingRunLength;
    if (version === 1)
        usingRunLength = false;
    else
        usingRunLength = bio.inputBit();
    var distEcoder = new AHuff(bio, 8);
    var lenEcoder = new AHuff(bio, 8);
    var outLen = bio.readValue(24);
    if (outLen > MAX_OUT_LEN)
        throw new EotError("MTX_ERROR", "LZCOMP outLen ".concat(outLen, " exceeds maximum (").concat(MAX_OUT_LEN, ")"));
    if (outLen > outCap)
        throw new EotError("MTX_ERROR", "".concat(OUTPUT_BUDGET_EXCEEDED, " (outLen ").concat(outLen, " > ").concat(outCap, ")"));
    var _b = setDistRange(outLen), DUP2 = _b.DUP2, DUP4 = _b.DUP4, DUP6 = _b.DUP6, NUM_SYMS = _b.NUM_SYMS;
    var symEcoder = new AHuff(bio, NUM_SYMS);
    var windowSize = PRELOAD_SIZE + outLen;
    var win = new Uint8Array(windowSize);
    initializeModel(win);
    var base = PRELOAD_SIZE;
    if (!usingRunLength) {
        var p = 0;
        while (p < outLen) {
            var symbol = symEcoder.readSymbol();
            if (symbol < 256)
                win[base + p++] = symbol;
            else if (symbol === DUP2) {
                win[base + p] = win[base + p - 2];
                p++;
            }
            else if (symbol === DUP4) {
                win[base + p] = win[base + p - 4];
                p++;
            }
            else if (symbol === DUP6) {
                win[base + p] = win[base + p - 6];
                p++;
            }
            else {
                var length = decodeLength(lenEcoder, symbol);
                var distance = decodeDistance(distEcoder, (symbol - 256 >>> LEN_WIDTH) + 1);
                if (distance >= MAX_2BYTE_DIST)
                    length++;
                if (!(length >= 0) || length > outLen - p)
                    throw new EotError("MTX_ERROR", "LZCOMP match length exceeds declared output size");
                var src = base + p - distance - length + 1;
                if (src < 0)
                    throw new EotError("MTX_ERROR", "LZCOMP match reaches before the start of the window");
                var dst = base + p;
                for (var end = dst + length; dst < end;)
                    win[dst++] = win[src++];
                p += length;
            }
        }
        return win.subarray(base, base + outLen);
    }
    var outBuf = new Uint8Array(outLen);
    var outIdx = 0;
    /**
    * Make room for `extra` more output bytes. Grows geometrically, but never
    * past the budget, and only fails when the bytes actually needed exceed it.
    */
    var ensureOut = function (extra) {
        var needed = outIdx + extra;
        if (needed <= outBuf.length)
            return;
        if (needed > outCap)
            throw new EotError("MTX_ERROR", OUTPUT_BUDGET_EXCEEDED);
        var tmp = new Uint8Array(Math.min(outCap, Math.max(needed, outBuf.length + (outBuf.length >>> 1) + 16)));
        tmp.set(outBuf.subarray(0, outIdx));
        outBuf = tmp;
    };
    var rleState = RLE_INITIAL;
    var rleEscape = 0;
    var rleCount = 0;
    /** Emit a single decompressed byte through the inline run-length decoder. */
    var emitByte = function (byte) {
        switch (rleState) {
            case RLE_INITIAL:
                rleEscape = byte;
                rleState = RLE_NORMAL;
                break;
            case RLE_NORMAL:
                if (byte === rleEscape)
                    rleState = RLE_SEEN_ESCAPE;
                else {
                    ensureOut(1);
                    outBuf[outIdx++] = byte;
                }
                break;
            case RLE_SEEN_ESCAPE:
                rleCount = byte;
                if (rleCount === 0) {
                    ensureOut(1);
                    outBuf[outIdx++] = rleEscape;
                    rleState = RLE_NORMAL;
                }
                else
                    rleState = RLE_NEED_BYTE;
                break;
            case RLE_NEED_BYTE:
                ensureOut(rleCount);
                outBuf.fill(byte, outIdx, outIdx + rleCount);
                outIdx += rleCount;
                rleState = RLE_NORMAL;
        }
    };
    var pos = 0;
    for (; pos < outLen;) {
        var symbol = symEcoder.readSymbol();
        var value = void 0;
        if (symbol < 256)
            value = symbol;
        else if (symbol === DUP2)
            value = win[base + pos - 2];
        else if (symbol === DUP4)
            value = win[base + pos - 4];
        else if (symbol === DUP6)
            value = win[base + pos - 6];
        else {
            var length = decodeLength(lenEcoder, symbol);
            var distance = decodeDistance(distEcoder, (symbol - 256 >>> LEN_WIDTH) + 1);
            if (distance >= MAX_2BYTE_DIST)
                length++;
            if (!(length >= 0) || length > outLen - pos)
                throw new EotError("MTX_ERROR", "LZCOMP match length exceeds declared output size");
            var start = base + pos - distance - length + 1;
            if (start < 0)
                throw new EotError("MTX_ERROR", "LZCOMP match reaches before the start of the window");
            for (var j = 0; j < length; j++) {
                value = win[start + j];
                win[base + pos] = value;
                pos++;
                emitByte(value);
            }
            continue;
        }
        win[base + pos] = value;
        pos++;
        emitByte(value);
    }
    if (pos !== outLen)
        throw new EotError("MTX_ERROR", "LZCOMP decode overran output: pos=".concat(pos, ", expected=").concat(outLen));
    if (rleState === RLE_SEEN_ESCAPE || rleState === RLE_NEED_BYTE)
        throw new EotError("MTX_ERROR", "LZCOMP run-length data ends inside an escape sequence");
    return outBuf.subarray(0, outIdx);
}
//#endregion
//#region src/sfnt-builder.ts
/** Log2 of the largest power of 2 <= n. */
function lgFloor(n) {
    var ret = 0;
    while (n > 1) {
        n = Math.floor(n / 2);
        ret++;
    }
    return ret;
}
/** Largest power of 2 <= n. */
function maxPow2(n) {
    return 1 << lgFloor(n);
}
/**
* Write the 12-byte SFNT offset (header) table.
*
* Layout (all big-endian):
*   scalarType   U32  — 0x00010000 for TrueType, 'OTTO' for CFF outlines
*   numTables    U16
*   searchRange  U16
*   entrySelector U16
*   rangeShift   U16
*/
function writeOffsetTable(ctr, out) {
    var _b;
    var numTables = ctr.tables.length;
    var searchRange = maxPow2(numTables) * 16;
    var entrySelector = lgFloor(numTables);
    var rangeShift = numTables * 16 - searchRange;
    out.writeU32((_b = ctr.scalarType) !== null && _b !== void 0 ? _b : 65536);
    out.writeU16(numTables);
    out.writeU16(searchRange);
    out.writeU16(entrySelector);
    out.writeU16(rangeShift);
}
/**
* Write the table directory — one 16-byte record per table.
*
* Each record (big-endian):
*   tag       4 bytes (ASCII)
*   checksum  U32
*   offset    U32
*   bufSize   U32
*/
function writeTableDirectory(ctr, out) {
    var e_8, _b;
    try {
        for (var _c = __values(__spreadArray([], __read(ctr.tables), false).sort(function (a, b) { return a.tag < b.tag ? -1 : a.tag > b.tag ? 1 : 0; })), _d = _c.next(); !_d.done; _d = _c.next()) {
            var table = _d.value;
            out.writeU8(table.tag.charCodeAt(0));
            out.writeU8(table.tag.charCodeAt(1));
            out.writeU8(table.tag.charCodeAt(2));
            out.writeU8(table.tag.charCodeAt(3));
            out.writeU32(table.checksum);
            out.writeU32(table.offset);
            out.writeU32(table.bufSize);
        }
    }
    catch (e_8_1) { e_8 = { error: e_8_1 }; }
    finally {
        try {
            if (_d && !_d.done && (_b = _c.return)) _b.call(_c);
        }
        finally { if (e_8) throw e_8.error; }
    }
}
/**
* Write table data to the output stream while computing its checksum.
*
* The data is treated as a sequence of big-endian U32 values.  If the
* table size is not a multiple of 4 the final chunk is zero-padded.
*
* Side-effects: sets `table.offset` and `table.checksum`.
*/
function writeTableCheckingSum(table, out) {
    table.offset = out.pos;
    var data = table.buf;
    var len = table.bufSize;
    var padded = len + 3 & -4;
    var buf = out.buf;
    var start = out.pos;
    buf.set(data.subarray(0, len), start);
    buf.fill(0, start + len, start + padded);
    if (table.tag === "head")
        buf.fill(0, start + 8, start + 12);
    var checksum = 0;
    for (var off = start, end = start + padded; off < end; off += 4) {
        var word = (buf[off] << 24 | buf[off + 1] << 16 | buf[off + 2] << 8 | buf[off + 3]) >>> 0;
        checksum = checksum + word >>> 0;
    }
    out.pos = start + padded;
    if (out.pos > out.size)
        out.size = out.pos;
    table.checksum = checksum;
}
function validateTables(ctr) {
    var e_9, _b;
    try {
        for (var _c = __values(ctr.tables), _d = _c.next(); !_d.done; _d = _c.next()) {
            var table = _d.value;
            if (!Number.isInteger(table.bufSize) || table.bufSize < 0 || table.bufSize > table.buf.length)
                throw new EotError("CORRUPT_FILE", "table ".concat(table.tag, " has an invalid buffer size"));
            if (table.tag === "head" && table.bufSize < 12)
                throw new EotError("MALFORMED_HEAD_TABLE", "head table is too short for checksumAdjustment");
        }
    }
    catch (e_9_1) { e_9 = { error: e_9_1 }; }
    finally {
        try {
            if (_d && !_d.done && (_b = _c.return)) _b.call(_c);
        }
        finally { if (e_9) throw e_9.error; }
    }
}
/** Size of the table directory: 16 bytes per table. */
function getTableDirectorySize(ctr) {
    return 16 * ctr.tables.length;
}
/** Total bytes required for the assembled font. */
function getRequiredSize(ctr) {
    var e_10, _b;
    var offsetTableSize = 12;
    var dirSize = getTableDirectorySize(ctr);
    var tableDataSize = 0;
    try {
        for (var _c = __values(ctr.tables), _d = _c.next(); !_d.done; _d = _c.next()) {
            var table = _d.value;
            tableDataSize += Math.ceil(table.bufSize / 4) * 4;
        }
    }
    catch (e_10_1) { e_10 = { error: e_10_1 }; }
    finally {
        try {
            if (_d && !_d.done && (_b = _c.return)) _b.call(_c);
        }
        finally { if (e_10) throw e_10.error; }
    }
    return offsetTableSize + dirSize + tableDataSize;
}
/**
* Assemble a complete TrueType font binary from an `SFNTContainer`.
*
* Steps:
*  1. Compute required output size and allocate a `Stream`.
*  2. Write the offset table (12 bytes).
*  3. Reserve space for the table directory.
*  4. Write each table's data, computing per-table checksums.
*  5. Seek back and write the table directory.
*  6. Compute whole-font checksum and patch the `head` table's
*     `checksumAdjustment` field.
*  7. Return the assembled bytes.
*/
function dumpContainer(ctr, options) {
    var e_11, _b, e_12, _c;
    validateTables(ctr);
    var requiredSize = getRequiredSize(ctr);
    var maxOutputBytes = options === null || options === void 0 ? void 0 : options.maxOutputBytes;
    if (maxOutputBytes !== void 0 && requiredSize > maxOutputBytes)
        throw new EotError("MTX_ERROR", "reconstructed font (".concat(requiredSize, " bytes) exceeds maxOutputBytes (").concat(maxOutputBytes, ")"));
    var out = new Stream(new Uint8Array(requiredSize), 0);
    writeOffsetTable(ctr, out);
    var dirOffset = out.pos;
    var dirSize = getTableDirectorySize(ctr);
    out.pos += dirSize;
    var totalChecksum = 0;
    try {
        for (var _d = __values(ctr.tables), _e = _d.next(); !_e.done; _e = _d.next()) {
            var table = _e.value;
            writeTableCheckingSum(table, out);
            totalChecksum = totalChecksum + table.checksum >>> 0;
        }
    }
    catch (e_11_1) { e_11 = { error: e_11_1 }; }
    finally {
        try {
            if (_e && !_e.done && (_b = _d.return)) _b.call(_d);
        }
        finally { if (e_11) throw e_11.error; }
    }
    var headTable;
    try {
        for (var _f = __values(ctr.tables), _g = _f.next(); !_g.done; _g = _f.next()) {
            var table = _g.value;
            if (table.tag === "head") {
                headTable = table;
                break;
            }
        }
    }
    catch (e_12_1) { e_12 = { error: e_12_1 }; }
    finally {
        try {
            if (_g && !_g.done && (_c = _f.return)) _c.call(_f);
        }
        finally { if (e_12) throw e_12.error; }
    }
    var afterTables = out.pos;
    out.pos = dirOffset;
    writeTableDirectory(ctr, out);
    var beginningLen = 12 + dirSize;
    var beginningChecksum = 0;
    var buf = out.buf;
    var beginningWords = Math.floor(beginningLen / 4);
    for (var i = 0; i < beginningWords; i++) {
        var off = i * 4;
        var word = (buf[off] << 24 | buf[off + 1] << 16 | buf[off + 2] << 8 | buf[off + 3]) >>> 0;
        beginningChecksum = beginningChecksum + word >>> 0;
    }
    totalChecksum = totalChecksum + beginningChecksum >>> 0;
    if (!headTable)
        throw new EotError("NO_HEAD_TABLE", "cannot assemble SFNT: container is missing a head table");
    var finalChecksum = 2981146554 - totalChecksum >>> 0;
    var adjOffset = headTable.offset + 8;
    buf[adjOffset] = finalChecksum >>> 24 & 255;
    buf[adjOffset + 1] = finalChecksum >>> 16 & 255;
    buf[adjOffset + 2] = finalChecksum >>> 8 & 255;
    buf[adjOffset + 3] = finalChecksum & 255;
    out.pos = afterTables;
    return out.buf.subarray(0, out.pos);
}
//#endregion
//#region src/mtx-decompress.ts
/**
* Top-level MicroType Express (MTX) decompression pipeline.
* Ported from libeot (MPL 2.0) — writeFontFile.c / liblzcomp.c
*
* Combines LZ decompression, CTF parsing, and SFNT assembly to
* produce a usable TrueType font from compressed MTX / EOT data.
*/
/** XOR key used for MTX encryption. */
var ENCRYPTION_KEY = 80;
/**
* Unpack an MTX blob into three LZCOMP-decompressed streams.
*
* MTX header layout (10 bytes, big-endian):
*   byte  0     : versionMagic
*   bytes 1–3   : copyLimit  (24-bit BE, informational only)
*   bytes 4–6   : offset2    (24-bit BE)
*   bytes 7–9   : offset3    (24-bit BE)
*
* The data following the header is split into three contiguous
* compressed blocks whose boundaries are determined by the offsets.
*
* @param data  Raw MTX data, i.e. the font-data region of the EOT container.
*              Real-world EOT files (Glyphicons, Font Awesome) decode as-is;
*              no BSGP stripping is needed.
* @param size  Total byte length of `data`.
* @param maxOutputBytes  Optional combined budget for the three decompressed
*              streams. Each stream is decoded against what is left of it,
*              so an oversized stream fails before its buffers are allocated.
* @returns An object containing the three decompressed byte arrays and
*          their respective sizes.
*/
function unpackMtx(data, size, maxOutputBytes) {
    if (maxOutputBytes === void 0) { maxOutputBytes = Infinity; }
    if (data.length < 10)
        throw new EotError("INSUFFICIENT_BYTES", "MTX data too small: header requires at least 10 bytes");
    if (!Number.isInteger(size))
        throw new EotError("MTX_ERROR", "MTX data size must be an integer: size=".concat(size));
    if (size < 10)
        throw new EotError("INSUFFICIENT_BYTES", "MTX data too small: header requires at least 10 bytes");
    if (size > data.length)
        throw new EotError("INSUFFICIENT_BYTES", "MTX data size exceeds the input buffer: size=".concat(size, ", data.length=").concat(data.length));
    var versionMagic = data[0];
    var offset2 = data[4] << 16 | data[5] << 8 | data[6];
    var offset3 = data[7] << 16 | data[8] << 8 | data[9];
    if (offset2 < 10 || offset3 < offset2 || offset3 > size)
        throw new EotError("MTX_ERROR", "MTX header offsets out of bounds: offset2=".concat(offset2, ", offset3=").concat(offset3, ", size=").concat(size));
    var offsets = [
        10,
        offset2,
        offset3
    ];
    var blockSizes = [
        Math.max(0, offset2 - 10),
        Math.max(0, offset3 - offset2),
        Math.max(0, size - offset3)
    ];
    var streams = [];
    var decompressedSizes = [];
    var budget = maxOutputBytes;
    for (var i = 0; i < 3; i++) {
        var decompressed = lzcompDecompress(data.subarray(offsets[i], offsets[i] + blockSizes[i]), blockSizes[i], versionMagic, budget);
        budget -= decompressed.length;
        streams.push(decompressed);
        decompressedSizes.push(decompressed.length);
    }
    return {
        streams: streams,
        sizes: decompressedSizes
    };
}
/** Default ceiling on the size of a reconstructed font (128 MiB). */
var DEFAULT_MAX_OUTPUT_BYTES = 134217728;
/**
* Decompress an MTX-compressed font (e.g. from an EOT wrapper) into a
* standard sfnt font binary (TrueType, or OpenType/CFF).
*
* @param fontData    Raw font bytes (MTX-compressed, optionally encrypted).
* @param options.encrypted   If `true`, XOR-decrypt with {@link ENCRYPTION_KEY}.
* @param options.compressed  If `false`, skip decompression and return the
*                            (possibly decrypted) data as-is.
* @param options.maxOutputBytes  Cap on the reconstructed font size.
* @param options.onWarn      Optional hook invoked with a message for each
*                            non-fatal diagnostic. The font is still produced.
* @returns A `Uint8Array` containing a valid sfnt font: TrueType, or OpenType/CFF (`OTTO`).
*/
function decompressMtx(fontData, options) {
    try {
        return decompressMtxUnchecked(fontData, options);
    }
    catch (err) {
        throw toEotError(err);
    }
}
function decompressMtxUnchecked(fontData, options) {
    var _b, _c, _d;
    var encrypted = (_b = options === null || options === void 0 ? void 0 : options.encrypted) !== null && _b !== void 0 ? _b : false;
    var compressed = (_c = options === null || options === void 0 ? void 0 : options.compressed) !== null && _c !== void 0 ? _c : true;
    var data;
    if (encrypted) {
        data = new Uint8Array(fontData.length);
        for (var i = 0; i < fontData.length; i++)
            data[i] = fontData[i] ^ ENCRYPTION_KEY;
    }
    else
        data = fontData;
    if (!compressed) {
        var tag = data.length >= 4 ? (data[0] << 24 | data[1] << 16 | data[2] << 8 | data[3]) >>> 0 : 0;
        if (tag !== 65536 && tag !== 1330926671 && tag !== 1953658213 && tag !== 1953784678)
            emitWarning(options, "WARN_NOT_SFNT", "uncompressed font data does not start with a recognised sfnt signature");
        return encrypted ? data : data.slice();
    }
    var maxOutputBytes = (_d = options === null || options === void 0 ? void 0 : options.maxOutputBytes) !== null && _d !== void 0 ? _d : 134217728;
    var streams;
    try {
        (streams = unpackMtx(data, data.length, maxOutputBytes).streams);
    }
    catch (err) {
        if (err instanceof EotError && err.message.startsWith("LZCOMP output exceeds maximum size budget"))
            throw new EotError("MTX_ERROR", "decompressed streams exceed maxOutputBytes (".concat(maxOutputBytes, ")"));
        throw err;
    }
    return dumpContainer(parseCTF(streams.map(function (buf) { return new Stream(buf, buf.length); }), {
        onWarn: options === null || options === void 0 ? void 0 : options.onWarn,
        onWarning: options === null || options === void 0 ? void 0 : options.onWarning
    }), { maxOutputBytes: maxOutputBytes });
}
/**
* Decompress an EOT-embedded font.
*
* This is a thin wrapper around {@link decompressMtx} that accepts explicit
* boolean parameters instead of an options object.
*
* @param fontData    Raw font bytes extracted from the EOT container.
* @param compressed  Whether the data is MTX-compressed.
* @param encrypted   Whether the data is XOR-encrypted.
* @returns A `Uint8Array` containing a valid sfnt font: TrueType, or OpenType/CFF (`OTTO`).
*/
function decompressEotFont(fontData, compressed, encrypted) {
    return decompressMtx(fontData, {
        encrypted: encrypted,
        compressed: compressed
    });
}
//#endregion
//#region src/eot.ts
/**
* EOT (Embedded OpenType) container parsing.
*
* Ported from libeot (MPL 2.0) — src/EOT.c. Parses the little-endian EOT
* header that wraps MTX-compressed font data, derives where the font data
* begins, and exposes the compression/encryption flags so callers no longer
* have to guess them. `eotToTtf` chains this into {@link decompressMtx} to
* turn a raw `.eot` file straight into a TrueType (or, for CFF outlines, OpenType) binary.
*
* Note: every field in the EOT header is LITTLE-endian, unlike the big-endian
* CTF/SFNT streams handled elsewhere in this library.
*
* @see http://www.w3.org/Submission/EOT/
*/
/** The font is a subset of the original. */
var TTEMBED_SUBSET = 1;
/** The font data is MTX-compressed. */
var TTEMBED_TTCOMPRESSED = 4;
/** The font data is XOR-obfuscated with key 0x50. */
var TTEMBED_XORENCRYPTDATA = 268435456;
/** Magic number appended after the code-page range fields (`"LP"` little-endian). */
var EOT_MAGIC = 20556;
/** fsType editing-permission mask (see {@link canLegallyEdit}). */
var EDITING_MASK = 8;
function readU16LE(bytes, at) {
    return bytes[at] | bytes[at + 1] << 8;
}
function readU32LE(bytes, at) {
    return (bytes[at] | bytes[at + 1] << 8 | bytes[at + 2] << 16 | bytes[at + 3] << 24) >>> 0;
}
/** Decode `count` UTF-16LE code units starting at `at` into a JS string. */
function decodeUtf16LE(bytes, at, byteLength) {
    var out = "";
    for (var i = 0; i < byteLength; i += 2)
        out += String.fromCharCode(readU16LE(bytes, at + i));
    return out;
}
/**
* A forward cursor over the header body. `limit` is the absolute index one past
* the last readable byte of the header (i.e. where the font data begins). The
* bound uses `>` (reading N bytes needs `pos + N <= limit`); libeot's
* `EOT_ENSURE_SCANNER` macro used `>=`, which is an off-by-one that spuriously
* rejects the exact-fit case — the string/array helpers there already use the
* correct `>`, so this matches libeot's intent.
*/
var Scanner = /** @class */ (function () {
    function class_2(bytes, start, limit) {
        this.bytes = bytes;
        this.limit = limit;
        this.pos = start;
    }
    class_2.prototype.ensure = function (n) {
        if (this.pos + n > this.limit)
            throw new EotError("INSUFFICIENT_BYTES", "EOT header truncated: need ".concat(n, " more byte(s) at offset ").concat(this.pos));
    };
    class_2.prototype.u16 = function () {
        this.ensure(2);
        var v = readU16LE(this.bytes, this.pos);
        this.pos += 2;
        return v;
    };
    class_2.prototype.u32 = function () {
        this.ensure(4);
        var v = readU32LE(this.bytes, this.pos);
        this.pos += 4;
        return v;
    };
    class_2.prototype.u8 = function () {
        this.ensure(1);
        return this.bytes[this.pos++];
    };
    class_2.prototype.take = function (n) {
        this.ensure(n);
        var slice = this.bytes.subarray(this.pos, this.pos + n);
        this.pos += n;
        return slice;
    };
    class_2.prototype.skip = function (n) {
        this.ensure(n);
        this.pos += n;
    };
    /** Read a length-prefixed (U16LE byte count) UTF-16LE string. */
    class_2.prototype.string = function () {
        this.ensure(2);
        var size = readU16LE(this.bytes, this.pos);
        this.pos += 2;
        if (size % 2 !== 0)
            throw new EotError("BOGUS_STRING_SIZE", "EOT string size ".concat(size, " is not a multiple of 2 (UTF-16)"));
        if (size === 0)
            return "";
        this.ensure(size);
        var s = decodeUtf16LE(this.bytes, this.pos, size);
        this.pos += size;
        return s;
    };
    /** Read a length-prefixed (U32LE byte count) raw byte array. */
    class_2.prototype.byteArray = function () {
        this.ensure(4);
        var size = readU32LE(this.bytes, this.pos);
        this.pos += 4;
        if (size === 0)
            return /* @__PURE__ */ new Uint8Array(0);
        return this.take(size);
    };
    return class_2;
}());
/** Sentinel thrown internally to drive the version-retry loop. */
var HeaderTooBig = /** @class */ (function (_super) {
    __extends(HeaderTooBig, _super);
    function HeaderTooBig() {
        return _super !== null && _super.apply(this, arguments) || this;
    }
    return HeaderTooBig;
}(Error));
/**
* Parse the version-specific portion of the header. Mirrors
* `EOTfillMetadataSpecifyingVersion`. Throws {@link HeaderTooBig} when the parse
* consumed less than the declared header (a signal to try a higher version),
* {@link EotError} with `InsufficientBytes` when it ran past the end (try lower),
* and `CorruptFile` / `BogusStringSize` as terminal failures.
*/
function parseBody(bytes, version, totalSize, fontDataSize) {
    var sc = new Scanner(bytes, 12, bytes.length - fontDataSize);
    var flags = sc.u32();
    var panose = sc.take(10).slice();
    var charset = sc.u8();
    var italic = sc.u8() !== 0;
    var weight = sc.u32();
    var permissions = sc.u16();
    if (sc.u16() !== EOT_MAGIC)
        throw new EotError("CORRUPT_FILE", "EOT magic number (0x504C) mismatch");
    var unicodeRange = [
        sc.u32(),
        sc.u32(),
        sc.u32(),
        sc.u32()
    ];
    var codePageRange = [sc.u32(), sc.u32()];
    var checkSumAdjustment = sc.u32();
    sc.skip(18);
    var familyName = sc.string();
    sc.skip(2);
    var styleName = sc.string();
    sc.skip(2);
    var versionName = sc.string();
    sc.skip(2);
    var fullName = sc.string();
    var rootString = "";
    var rootStringChecksum = 0;
    var eudcCodePage = 0;
    var eudcFlags = 0;
    var eudcFontData;
    if (version > 1) {
        sc.skip(2);
        rootString = sc.string();
        if (version === 3) {
            rootStringChecksum = sc.u32();
            eudcCodePage = sc.u32();
            sc.skip(2);
            var signatureSize = sc.u16();
            sc.skip(signatureSize);
            eudcFlags = sc.u32();
            var eudc = sc.byteArray();
            eudcFontData = eudc.length > 0 ? eudc.slice() : void 0;
        }
    }
    var fontDataOffset = sc.pos;
    if (fontDataOffset < totalSize - fontDataSize)
        throw new HeaderTooBig();
    return {
        version: version,
        flags: flags,
        panose: panose,
        charset: charset,
        italic: italic,
        weight: weight,
        permissions: permissions,
        unicodeRange: unicodeRange,
        codePageRange: codePageRange,
        checkSumAdjustment: checkSumAdjustment,
        familyName: familyName,
        styleName: styleName,
        versionName: versionName,
        fullName: fullName,
        rootString: rootString,
        rootStringChecksum: rootStringChecksum,
        eudcCodePage: eudcCodePage,
        eudcFlags: eudcFlags,
        eudcFontData: eudcFontData,
        totalSize: totalSize,
        fontDataSize: fontDataSize,
        fontDataOffset: fontDataOffset
    };
}
var VERSION_MAGIC = {
    65536: 1,
    131073: 2,
    131074: 3
};
/** Normalise {@link BinaryInput} to a `Uint8Array` view (no copy). */
function toUint8Array(input) {
    if (input instanceof Uint8Array)
        return input;
    if (ArrayBuffer.isView(input))
        return new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
    return new Uint8Array(input);
}
/**
* Parse the metadata of an EOT container.
*
* Reproduces libeot's `EOTfillMetadata`, including the version-retry loop that
* copes with EOT files whose declared version disagrees with their actual
* layout. On a corrected version {@link EotMetadata.badVersion} is set rather
* than throwing (libeot returns the recoverable `EOT_WARN_BAD_VERSION`).
*
* @param input Raw `.eot` file bytes (`Uint8Array`, `ArrayBuffer` or any typed-array view).
* @throws {EotError} on a corrupt or truncated container.
*/
function parseEotMetadata(input) {
    var bytes = toUint8Array(input);
    if (bytes.length < 8)
        throw new EotError("INSUFFICIENT_BYTES", "EOT file too small (need at least 8 bytes)");
    var totalSize = readU32LE(bytes, 0);
    var fontDataSize = readU32LE(bytes, 4);
    if (fontDataSize > totalSize)
        throw new EotError("CORRUPT_FILE", "EOT font data size exceeds declared total size");
    var metadataLength = totalSize - fontDataSize;
    if (bytes.length < metadataLength)
        throw new EotError("INSUFFICIENT_BYTES", "EOT file shorter than its declared metadata length (".concat(metadataLength, ")"));
    if (bytes.length < 12)
        throw new EotError("INSUFFICIENT_BYTES", "EOT file too small for a version field");
    var versionMagic = readU32LE(bytes, 8);
    var codedVersion = VERSION_MAGIC[versionMagic];
    if (codedVersion === void 0)
        throw new EotError("CORRUPT_FILE", "unrecognized EOT version magic 0x".concat(versionMagic.toString(16)));
    if (12 + fontDataSize > bytes.length)
        throw new EotError("CORRUPT_FILE", "EOT font data extends past end of file");
    var tryVersion = codedVersion;
    var bumpedUp = false;
    var knockedDown = false;
    while (true)
        try {
            var body = parseBody(bytes, tryVersion, totalSize, fontDataSize);
            var flags = body.flags;
            return __assign(__assign({}, body), { compressed: (flags & 4) !== 0, encrypted: (flags & TTEMBED_XORENCRYPTDATA) !== 0, subset: (flags & 1) !== 0, badVersion: tryVersion !== codedVersion });
        }
        catch (err) {
            if (err instanceof HeaderTooBig) {
                if (knockedDown || tryVersion === 3)
                    throw new EotError("CORRUPT_FILE", "EOT header inconsistent across all versions");
                knockedDown = false;
                bumpedUp = true;
                tryVersion = tryVersion + 1;
                continue;
            }
            if (err instanceof EotError && err.code === "INSUFFICIENT_BYTES") {
                if (bumpedUp || tryVersion === 1)
                    throw new EotError("CORRUPT_FILE", "EOT header inconsistent across all versions");
                knockedDown = true;
                bumpedUp = false;
                tryVersion = tryVersion - 1;
                continue;
            }
            throw err;
        }
}
/**
* Inspect the protection indicators declared by an EOT container.
*
* This reports EOT's XOR transform independently from `fsType` embedding
* permissions and RootString URL restrictions. EOT has no password-based
* protection mechanism; this cannot determine whether an outer file or
* document containing the EOT is password-protected.
*
* @param input Raw `.eot` file bytes (`Uint8Array`, `ArrayBuffer` or any typed-array view).
* @throws {EotError} on a corrupt or truncated container.
*/
function inspectEotProtection(input) {
    var metadata = parseEotMetadata(input);
    return {
        encryption: metadata.encrypted ? "xor-0x50" : "none",
        passwordProtection: "not_supported_by_eot",
        embeddingPermissions: metadata.permissions,
        rootString: metadata.rootString
    };
}
/**
* Decode a raw EOT container straight into a font binary: TrueType (.ttf), or OpenType/CFF (`OTTO`) when the outlines are CFF.
*
* Parses the header, locates and slices the embedded font data, and runs it
* through {@link decompressMtx} using the container's own
* compressed/encrypted flags. This is the drop-in equivalent of libeot's
* `EOT2ttf_*` entry points.
*
* @param input Raw `.eot` file bytes (`Uint8Array`, `ArrayBuffer` or any typed-array view).
* @returns The reconstructed font (TrueType, or OpenType/CFF).
* @param options.maxOutputBytes Cap on the reconstructed font size (default 128 MiB).
* @param options.onWarn Called for recovered header-version mismatches and
* non-fatal decompression diagnostics.
* @throws {EotError} on a corrupt container or during decompression.
*/
function eotToTtf(input, options) {
    var bytes = toUint8Array(input);
    var meta = parseEotMetadata(bytes);
    if (meta.badVersion)
        emitWarning(options, "WARN_BAD_VERSION", "EOT header version disagrees with its layout; decoded as version ".concat(meta.version));
    return decompressMtx(bytes.subarray(meta.fontDataOffset, meta.fontDataOffset + meta.fontDataSize), {
        compressed: meta.compressed,
        encrypted: meta.encrypted,
        onWarn: options === null || options === void 0 ? void 0 : options.onWarn,
        onWarning: options === null || options === void 0 ? void 0 : options.onWarning,
        maxOutputBytes: options === null || options === void 0 ? void 0 : options.maxOutputBytes
    });
}
/**
* Whether the font's embedding permissions allow editing.
*
* Mirrors libeot's `EOTcanLegallyEdit`. The upstream author asks that callers
* reflect before circumventing this: installable-permission fonts (`fsType`
* 0) and editable-embedding fonts may be edited; others may not.
*/
function canLegallyEdit(metadata) {
    return metadata.permissions === 0 || (metadata.permissions & EDITING_MASK) !== 0;
}
/**
* Promise-returning variant of {@link eotToTtf} that also accepts a `Blob`/`File`
* (convenient in browsers). Decoding itself still runs synchronously once the
* bytes are available.
*/
function eotToTtfAsync(input, options) {
    return __awaiter(this, void 0, void 0, function () {
        var _b, _c;
        return __generator(this, function (_d) {
            switch (_d.label) {
                case 0:
                    _b = eotToTtf;
                    if (!("arrayBuffer" in input && typeof input.arrayBuffer === "function")) return [3 /*break*/, 2];
                    return [4 /*yield*/, input.arrayBuffer()];
                case 1:
                    _c = _d.sent();
                    return [3 /*break*/, 3];
                case 2:
                    _c = input;
                    _d.label = 3;
                case 3: return [2 /*return*/, _b.apply(void 0, [_c, options])];
            }
        });
    });
}
//#endregion
exports.DEFAULT_MAX_OUTPUT_BYTES = DEFAULT_MAX_OUTPUT_BYTES;
exports.EOT_WARN = EOT_WARN;
exports.EotError = EotError;
exports.EotErrorCode = EotErrorCode;
exports.Stream = Stream;
exports.TTEMBED_SUBSET = TTEMBED_SUBSET;
exports.TTEMBED_TTCOMPRESSED = TTEMBED_TTCOMPRESSED;
exports.TTEMBED_XORENCRYPTDATA = TTEMBED_XORENCRYPTDATA;
exports.canLegallyEdit = canLegallyEdit;
exports.decompressEotFont = decompressEotFont;
exports.decompressMtx = decompressMtx;
exports.dumpContainer = dumpContainer;
exports.eotToTtf = eotToTtf;
exports.eotToTtfAsync = eotToTtfAsync;
exports.inspectEotProtection = inspectEotProtection;
exports.parseCTF = parseCTF;
exports.parseEotMetadata = parseEotMetadata;
exports.unpackMtx = unpackMtx;
