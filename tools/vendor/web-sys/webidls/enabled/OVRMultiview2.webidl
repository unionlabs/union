/* -*- Mode: IDL; tab-width: 4; indent-tabs-mode: nil; c-basic-offset: 4 -*- */
/*
 * The source for this IDL is found at https://www.khronos.org/registry/webgl/extensions/OVR_multiview2/
 */
[Exposed=(Window,Worker), LegacyNoInterfaceObject]
interface OVR_multiview2 {
    const GLenum FRAMEBUFFER_ATTACHMENT_TEXTURE_NUM_VIEWS_OVR = 0x9630;
    const GLenum FRAMEBUFFER_ATTACHMENT_TEXTURE_BASE_VIEW_INDEX_OVR = 0x9632;
    const GLenum MAX_VIEWS_OVR = 0x9631;
    const GLenum FRAMEBUFFER_INCOMPLETE_VIEW_TARGETS_OVR = 0x9633;

    undefined framebufferTextureMultiviewOVR(GLenum target, GLenum attachment, WebGLTexture? texture, GLint level, GLint baseViewIndex, GLsizei numViews);
};
