//! Bindings to functions contained within lz4frame.h

#![allow(unstable)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

extern crate libc;

use libc::{c_uint, c_int, size_t, c_char, c_void};

use std::ffi::CString;

pub use self::types::*;

pub mod types;

// unfortunately this constant is defined as a macro so we can't import and and will need to keep
// it updated manually
pub const LZ4F_VERSION: c_uint = 100;


extern {

    /**************************************
     * Error management
     * ************************************/

    // unsigned    LZ4F_isError(LZ4F_errorCode_t code);
    pub fn LZ4F_isError(code: size_t) -> c_uint;

    // const char* LZ4F_getErrorName(LZ4F_errorCode_t code);
    /* return error code string; useful for debugging */
    pub fn LZ4F_getErrorName(code: size_t) -> *const c_char;


    /***********************************
     * Simple compression function
     * *********************************/

    // size_t LZ4F_compressFrameBound(size_t srcSize, const LZ4F_preferences_t* preferencesPtr);
    pub fn LZ4F_compressFrameBound(srcSize: size_t, prefsPtr: *const FramePreferences) -> size_t;

    // size_t LZ4F_compressFrame(void* dstBuffer, size_t dstMaxSize, const void* srcBuffer,
    //                           size_t srcSize, const LZ4F_preferences_t* preferencesPtr);
    /* LZ4F_compressFrame()
     * Compress an entire srcBuffer into a valid LZ4 frame, as defined by specification v1.5
     * The most important rule is that dstBuffer MUST be large enough (dstMaxSize) to ensure
     * compression completion even in worst case. You can get the minimum value of dstMaxSize by
     * using LZ4F_compressFrameBound()
     * If this condition is not respected, LZ4F_compressFrame() will fail (result is an errorCode)
     * The LZ4F_preferences_t structure is optional : you can provide NULL as argument. All
     * preferences will be set to default.
     * The result of the function is the number of bytes written into dstBuffer.
     * The function outputs an error code if it fails (can be tested using LZ4F_isError())
     */
    pub fn LZ4F_compressFrame(dstBuffer: *mut c_void,
                              dstMaxSize: size_t,
                              srcBuffer: *const c_void,
                              srcSize: size_t,
                              prefsPtr: *const FramePreferences) -> size_t;

   
    /**********************************
     * Advanced compression functions
     * ********************************/

    /* Resource Management */

    // LZ4F_errorCode_t LZ4F_createCompressionContext(LZ4F_compressionContext_t* cctxPtr,
    //                                                unsigned version);
    /* LZ4F_createCompressionContext() :
     * The first thing to do is to create a compressionContext object, which will be used in all
     * compression operations.
     * This is achieved using LZ4F_createCompressionContext(), which takes as argument a version
     * and an LZ4F_preferences_t structure.
     * The version provided MUST be LZ4F_VERSION. It is intended to track potential version
     * differences between different binaries.
     * The function will provide a pointer to a fully allocated LZ4F_compressionContext_t object.
     * If the result LZ4F_errorCode_t is not zero, there was an error during context creation.
     * Object can release its memory using LZ4F_freeCompressionContext();
     */
    pub fn LZ4F_createCompressionContext(cctxPtr: *mut Context, version: c_uint) -> FrameErrorCode;

    // LZ4F_errorCode_t LZ4F_freeCompressionContext(LZ4F_compressionContext_t cctx);
    pub fn LZ4F_freeCompressionContext(cctx: Context) -> FrameErrorCode;

    /* Compression */

    // size_t LZ4F_compressBegin(LZ4F_compressionContext_t cctx, void* dstBuffer,
    //                           size_t dstMaxSize, const LZ4F_preferences_t* prefsPtr);
    /* LZ4F_compressBegin() :
     * will write the frame header into dstBuffer.
     * dstBuffer must be large enough to accommodate a header (dstMaxSize). Maximum header size
     * is 15 bytes.
     * The LZ4F_preferences_t structure is optional : you can provide NULL as argument, all
     * preferences will then be set to default.
     * The result of the function is the number of bytes written into dstBuffer for the header
     * or an error code (can be tested using LZ4F_isError())
     */
    pub fn LZ4F_compressBegin(cctx: Context,
                              dstBuffer: *mut c_void,
                              dstMaxSize: size_t,
                              prefsPtr: *const FramePreferences) -> size_t;


    // size_t LZ4F_compressBound(size_t srcSize, const LZ4F_preferences_t* prefsPtr);
    /* LZ4F_compressBound() :
     * Provides the minimum size of Dst buffer given srcSize to handle worst case situations.
     * prefsPtr is optional : you can provide NULL as argument, all preferences will then be set to
     * default.
     * Note that different preferences will produce in different results.
     */
    pub fn LZ4F_compressBound(srcSize: size_t, prefsPtr: *const FramePreferences) -> size_t;

    // size_t LZ4F_compressUpdate(LZ4F_compressionContext_t cctx, void* dstBuffer,
    //                            size_t dstMaxSize, const void* srcBuffer, size_t srcSize,
    //                            const LZ4F_compressOptions_t* cOptPtr);
    /* LZ4F_compressUpdate()
     * LZ4F_compressUpdate() can be called repetitively to compress as much data as necessary.
     * The most important rule is that dstBuffer MUST be large enough (dstMaxSize) to ensure
     * compression completion even in worst case.
     * If this condition is not respected, LZ4F_compress() will fail (result is an errorCode)
     * You can get the minimum value of dstMaxSize by using LZ4F_compressBound()
     * The LZ4F_compressOptions_t structure is optional : you can provide NULL as argument.
     * The result of the function is the number of bytes written into dstBuffer : it can be zero,
     * meaning input data was just buffered.
     * The function outputs an error code if it fails (can be tested using LZ4F_isError())
     */
    pub fn LZ4F_compressUpdate(cctx: Context,
                               dstBuffer: *mut c_void,
                               dstMaxSize: size_t,
                               srcBuffer: *const c_void,
                               srcSize: size_t,
                               cOptPtr: *const FrameCompressOptions) -> size_t;

    // size_t LZ4F_flush(LZ4F_compressionContext_t cctx, void* dstBuffer, size_t dstMaxSize,
    //                   const LZ4F_compressOptions_t* cOptPtr);

    /* LZ4F_flush()
     * Should you need to generate compressed data immediately, without waiting for the current
     * block to be filled,
     * you can call LZ4_flush(), which will immediately compress any remaining data buffered within
     * cctx.
     * Note that dstMaxSize must be large enough to ensure the operation will be successful.
     * LZ4F_compressOptions_t structure is optional : you can provide NULL as argument.
     * The result of the function is the number of bytes written into dstBuffer
     * (it can be zero, this means there was no data left within cctx)
     * The function outputs an error code if it fails (can be tested using LZ4F_isError())
     */    
    pub fn LZ4F_flush(cctx: Context,
                      dstBuffer: *mut c_void,
                      dstMaxSize: size_t,
                      cOptPtr: *const FrameCompressOptions) -> size_t;

    // size_t LZ4F_compressEnd(LZ4F_compressionContext_t cctx, void* dstBuffer, size_t dstMaxSize,
    //                         const LZ4F_compressOptions_t* cOptPtr);
    /* LZ4F_compressEnd()
     * When you want to properly finish the compressed frame, just call LZ4F_compressEnd().
     * It will flush whatever data remained within compressionContext (like LZ4_flush())
     * but also properly finalize the frame, with an endMark and a checksum.
     * The result of the function is the number of bytes written into dstBuffer
     * (necessarily >= 4 (endMark size))
     * The function outputs an error code if it fails (can be tested using LZ4F_isError())
     * The LZ4F_compressOptions_t structure is optional : you can provide NULL as argument.
     * A successful call to LZ4F_compressEnd() makes cctx available again for future compression
     * work.
     */
    pub fn LZ4F_compressEnd(cctx: Context,
                            dstBuffer: *mut c_void,
                            dstMaxSize: size_t,
                            cOptPtr: *const FrameCompressOptions) -> size_t;


    /***********************************
     * Decompression functions
     * *********************************/

    /* Resource management */

    // LZ4F_errorCode_t LZ4F_createDecompressionContext(LZ4F_decompressionContext_t* dctxPtr,
    //                                                  unsigned version);
    /* LZ4F_createDecompressionContext() :
     * The first thing to do is to create an LZ4F_decompressionContext_t object, which will be used
     * in all decompression operations.
     * This is achieved using LZ4F_createDecompressionContext().
     * The version provided MUST be LZ4F_VERSION. It is intended to track potential breaking
     * differences between different versions.
     * The function will provide a pointer to a fully allocated and initialized
     * LZ4F_decompressionContext_t object.
     * The result is an errorCode, which can be tested using LZ4F_isError().
     * dctx memory can be released using LZ4F_freeDecompressionContext();
     */
    pub fn LZ4F_createDecompressionContext(dctxPtr: *mut Context, version: c_uint) -> FrameErrorCode;

    // LZ4F_errorCode_t LZ4F_freeDecompressionContext(LZ4F_decompressionContext_t dctx);
    pub fn LZ4F_freeDecompressionContext(dctx: Context) -> FrameErrorCode;


    /* Decompression */

    // size_t LZ4F_getFrameInfo(LZ4F_decompressionContext_t dctx,
    //                          LZ4F_frameInfo_t* frameInfoPtr,
    //                          const void* srcBuffer, size_t* srcSizePtr);
    /* LZ4F_getFrameInfo()
     * This function decodes frame header information, such as blockSize.
     * It is optional : you could start by calling directly LZ4F_decompress() instead.
     * The objective is to extract header information without starting decompression, typically for
     * allocation purposes.
     * The function will work only if srcBuffer points at the beginning of the frame,
     * and *srcSizePtr is large enough to decode the whole header (typically, between 7 & 15 bytes).
     * The result is copied into an LZ4F_frameInfo_t structure, which is pointed by frameInfoPtr,
     * and must be already allocated.
     * LZ4F_getFrameInfo() can also be used *after* starting decompression, on a valid
     * LZ4F_decompressionContext_t.
     * The number of bytes read from srcBuffer will be provided within *srcSizePtr
     * (necessarily <= original value).
     * It is basically the frame header size.
     * You are expected to resume decompression from where it stopped (srcBuffer + *srcSizePtr)
     * The function result is an hint of how many srcSize bytes LZ4F_decompress() expects for next
     * call, or an error code which can be tested using LZ4F_isError().
     */
    pub fn LZ4F_getFrameInfo(dctx: Context,
                             frameInfoPtr: *mut FrameInfo,
                             srcBuffer: *const c_void,
                             srcSizePtr: *mut size_t) -> size_t;

    // size_t LZ4F_decompress(LZ4F_decompressionContext_t dctx,
    //                        void* dstBuffer, size_t* dstSizePtr,
    //                        const void* srcBuffer, size_t* srcSizePtr,
    //                        const LZ4F_decompressOptions_t* dOptPtr);
    /* LZ4F_decompress()
     * Call this function repetitively to regenerate data compressed within srcBuffer.
     * The function will attempt to decode *srcSizePtr bytes from srcBuffer, into dstBuffer of
     * maximum size *dstSizePtr.
     *
     * The number of bytes regenerated into dstBuffer will be provided within *dstSizePtr
     * (necessarily <= original value).
     *
     * The number of bytes read from srcBuffer will be provided within *srcSizePtr
     * (necessarily <= original value).
     * If number of bytes read is < number of bytes provided, then decompression operation is not
     * completed.
     * It typically happens when dstBuffer is not large enough to contain all decoded data.
     * LZ4F_decompress() must be called again, starting from where it stopped
     * (srcBuffer + *srcSizePtr)
     * The function will check this condition, and refuse to continue if it is not respected.
     *
     * dstBuffer is supposed to be flushed between each call to the function, since its content
     * will be overwritten.
     * dst arguments can be changed at will with each consecutive call to the function.
     *
     * The function result is an hint of how many srcSize bytes LZ4F_decompress() expects for next
     * call.
     * Schematically, it's the size of the current (or remaining) compressed block + header of next
     * block.
     * Respecting the hint provides some boost to performance, since it does skip intermediate
     * buffers.
     * This is just a hint, you can always provide any srcSize you want.
     * When a frame is fully decoded, the function result will be 0 (no more data expected).
     * If decompression failed, function result is an error code, which can be tested using
     * LZ4F_isError().
     *
     * After a frame is fully decoded, dctx can be used again to decompress another frame.
     */
    pub fn LZ4F_decompress(dctx: Context,
                           dstBuffer: *mut c_void,
                           dstSizePtr: *mut size_t,
                           srcBuffer: *const c_void,
                           srcSizePtr: *mut size_t,
                           dOptPtr: *const FrameDecompressOptions) -> size_t;
}







