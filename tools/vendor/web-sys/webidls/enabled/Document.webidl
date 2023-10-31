/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * https://dom.spec.whatwg.org/#interface-document
 * https://html.spec.whatwg.org/multipage/dom.html#the-document-object
 * https://html.spec.whatwg.org/multipage/obsolete.html#other-elements%2C-attributes-and-apis
 * https://fullscreen.spec.whatwg.org/#api
 * https://w3c.github.io/pointerlock/#extensions-to-the-document-interface
 * https://w3c.github.io/pointerlock/#extensions-to-the-documentorshadowroot-mixin
 * https://w3c.github.io/page-visibility/#extensions-to-the-document-interface
 * https://drafts.csswg.org/cssom/#extensions-to-the-document-interface
 * https://drafts.csswg.org/cssom-view/#extensions-to-the-document-interface
 */

/*TODO
interface WindowProxy;
interface nsISupports;
interface URI;
interface nsIDocShell;
interface nsILoadGroup;
*/

enum VisibilityState { "hidden", "visible" };

/* https://dom.spec.whatwg.org/#dictdef-elementcreationoptions */
dictionary ElementCreationOptions {
  DOMString is;

  [ChromeOnly]
  DOMString pseudo;
};

/* https://dom.spec.whatwg.org/#interface-document */
[Constructor]
interface Document : Node {
  [Throws]
  readonly attribute DOMImplementation implementation;
  [Pure, Throws, BinaryName="documentURIFromJS", NeedsCallerType]
  readonly attribute DOMString URL;
  [Pure, Throws, BinaryName="documentURIFromJS", NeedsCallerType]
  readonly attribute DOMString documentURI;
  [Pure]
  readonly attribute DOMString compatMode;
  [Pure]
  readonly attribute DOMString characterSet;
  [Pure,BinaryName="characterSet"]
  readonly attribute DOMString charset; // legacy alias of .characterSet
  [Pure,BinaryName="characterSet"]
  readonly attribute DOMString inputEncoding; // legacy alias of .characterSet
  [Pure]
  readonly attribute DOMString contentType;

  [Pure]
  readonly attribute DocumentType? doctype;
  [Pure]
  readonly attribute Element? documentElement;

  [Pure]
  HTMLCollection getElementsByTagName(DOMString localName);
  [Pure, Throws]
  HTMLCollection getElementsByTagNameNS(DOMString? namespace, DOMString localName);
  [Pure]
  HTMLCollection getElementsByClassName(DOMString classNames);
  [Pure]
  Element? getElementById(DOMString elementId);

  [CEReactions, NewObject, Throws]
  Element createElement(DOMString localName, optional (ElementCreationOptions or DOMString) options);
  [CEReactions, NewObject, Throws]
  Element createElementNS(DOMString? namespace, DOMString qualifiedName, optional (ElementCreationOptions or DOMString) options);
  [NewObject]
  DocumentFragment createDocumentFragment();
  [NewObject]
  Text createTextNode(DOMString data);
  [NewObject]
  Comment createComment(DOMString data);
  [NewObject, Throws]
  ProcessingInstruction createProcessingInstruction(DOMString target, DOMString data);

  [CEReactions, Throws]
  Node importNode(Node node, optional boolean deep = false);
  [CEReactions, Throws]
  Node adoptNode(Node node);

  [NewObject, Throws, NeedsCallerType]
  Event createEvent(DOMString interface);

  [NewObject, Throws]
  Range createRange();

  // NodeFilter.SHOW_ALL = 0xFFFFFFFF
  [NewObject, Throws]
  NodeIterator createNodeIterator(Node root, optional unsigned long whatToShow = 0xFFFFFFFF, optional NodeFilter? filter = null);
  [NewObject, Throws]
  TreeWalker createTreeWalker(Node root, optional unsigned long whatToShow = 0xFFFFFFFF, optional NodeFilter? filter = null);

  // NEW
  // No support for prepend/append yet
  // undefined prepend((Node or DOMString)... nodes);
  // undefined append((Node or DOMString)... nodes);

  // These are not in the spec, but leave them for now for backwards compat.
  // So sort of like Gecko extensions
  [NewObject, Throws]
  CDATASection createCDATASection(DOMString data);
  [NewObject, Throws]
  Attr createAttribute(DOMString name);
  [NewObject, Throws]
  Attr createAttributeNS(DOMString? namespace, DOMString name);
};

// https://html.spec.whatwg.org/multipage/dom.html#the-document-object
partial interface Document {
  [PutForwards=href, Unforgeable] readonly attribute Location? location;
  //(HTML only)         attribute DOMString domain;
  readonly attribute DOMString referrer;
  //(HTML only)         attribute DOMString cookie;
  readonly attribute DOMString lastModified;
  readonly attribute DOMString readyState;

  // DOM tree accessors
  //(Not proxy yet)getter object (DOMString name);
  [CEReactions, SetterThrows, Pure]
           attribute DOMString title;
  [CEReactions, Pure]
           attribute DOMString dir;
  [CEReactions, Pure, SetterThrows]
           attribute HTMLElement? body;
  [Pure]
  readonly attribute HTMLHeadElement? head;
  [SameObject] readonly attribute HTMLCollection images;
  [SameObject] readonly attribute HTMLCollection embeds;
  [SameObject] readonly attribute HTMLCollection plugins;
  [SameObject] readonly attribute HTMLCollection links;
  [SameObject] readonly attribute HTMLCollection forms;
  [SameObject] readonly attribute HTMLCollection scripts;
  [Pure]
  NodeList getElementsByName(DOMString elementName);

  //(Not implemented)readonly attribute DOMElementMap cssElementMap;

  // dynamic markup insertion
  //(HTML only)Document open(optional DOMString type, optional DOMString replace);
  //(HTML only)WindowProxy open(DOMString url, DOMString name, DOMString features, optional boolean replace);
  //(HTML only)undefined close();
  //(HTML only)undefined write(DOMString... text);
  //(HTML only)undefined writeln(DOMString... text);

  // user interaction
  [Pure]
  readonly attribute WindowProxy? defaultView;
  [Throws]
  boolean hasFocus();
  //(HTML only)         attribute DOMString designMode;
  //(HTML only)boolean execCommand(DOMString commandId);
  //(HTML only)boolean execCommand(DOMString commandId, boolean showUI);
  //(HTML only)boolean execCommand(DOMString commandId, boolean showUI, DOMString value);
  //(HTML only)boolean queryCommandEnabled(DOMString commandId);
  //(HTML only)boolean queryCommandIndeterm(DOMString commandId);
  //(HTML only)boolean queryCommandState(DOMString commandId);
  //(HTML only)boolean queryCommandSupported(DOMString commandId);
  //(HTML only)DOMString queryCommandValue(DOMString commandId);
  //(Not implemented)readonly attribute HTMLCollection commands;

  // special event handler IDL attributes that only apply to Document objects
  [LenientThis] attribute EventHandler onreadystatechange;

  // Gecko extensions?
                attribute EventHandler onbeforescriptexecute;
                attribute EventHandler onafterscriptexecute;

                [Pref="dom.select_events.enabled"]
                attribute EventHandler onselectionchange;

  /**
   * Returns the script element whose script is currently being processed.
   *
   * @see <https://developer.mozilla.org/en/DOM/document.currentScript>
   */
  [Pure]
  readonly attribute Element? currentScript;
  /**
   * Release the current mouse capture if it is on an element within this
   * document.
   *
   * @see <https://developer.mozilla.org/en/DOM/document.releaseCapture>
   */
  undefined releaseCapture();

  [ChromeOnly]
  readonly attribute URI? documentURIObject;

  /**
   * Current referrer policy - one of the REFERRER_POLICY_* constants
   * from nsIHttpChannel.
   */
  [ChromeOnly]
  readonly attribute unsigned long referrerPolicy;

};

// https://html.spec.whatwg.org/multipage/obsolete.html#other-elements%2C-attributes-and-apis
partial interface Document {
  //(HTML only)[CEReactions] attribute [TreatNullAs=EmptyString] DOMString fgColor;
  //(HTML only)[CEReactions] attribute [TreatNullAs=EmptyString] DOMString linkColor;
  //(HTML only)[CEReactions] attribute [TreatNullAs=EmptyString] DOMString vlinkColor;
  //(HTML only)[CEReactions] attribute [TreatNullAs=EmptyString] DOMString alinkColor;
  //(HTML only)[CEReactions] attribute [TreatNullAs=EmptyString] DOMString bgColor;

  [SameObject] readonly attribute HTMLCollection anchors;
  [SameObject] readonly attribute HTMLCollection applets;

  //(HTML only)undefined clear();
  //(HTML only)undefined captureEvents();
  //(HTML only)undefined releaseEvents();

  //(HTML only)[SameObject] readonly attribute HTMLAllCollection all;
};

// https://fullscreen.spec.whatwg.org/#api
partial interface Document {
  // Note: Per spec the 'S' in these two is lowercase, but the "Moz"
  // versions have it uppercase.
  [LenientSetter, Unscopable, Func="nsDocument::IsUnprefixedFullscreenEnabled"]
  readonly attribute boolean fullscreen;
  [LenientSetter, Func="nsDocument::IsUnprefixedFullscreenEnabled", NeedsCallerType]
  readonly attribute boolean fullscreenEnabled;

  [Func="nsDocument::IsUnprefixedFullscreenEnabled"]
  undefined exitFullscreen();

  // Events handlers
  [Func="nsDocument::IsUnprefixedFullscreenEnabled"]
  attribute EventHandler onfullscreenchange;
  [Func="nsDocument::IsUnprefixedFullscreenEnabled"]
  attribute EventHandler onfullscreenerror;
};

// https://w3c.github.io/pointerlock/#extensions-to-the-document-interface
// https://w3c.github.io/pointerlock/#extensions-to-the-documentorshadowroot-mixin
partial interface Document {
  undefined exitPointerLock();

  // Event handlers
  attribute EventHandler onpointerlockchange;
  attribute EventHandler onpointerlockerror;
};

// https://w3c.github.io/page-visibility/#extensions-to-the-document-interface
partial interface Document {
  readonly attribute boolean hidden;
  readonly attribute VisibilityState visibilityState;
           attribute EventHandler onvisibilitychange;
};

// https://drafts.csswg.org/cssom/#extensions-to-the-document-interface
partial interface Document {
    attribute DOMString? selectedStyleSheetSet;
    readonly attribute DOMString? lastStyleSheetSet;
    readonly attribute DOMString? preferredStyleSheetSet;
    [Constant]
    readonly attribute DOMStringList styleSheetSets;
    undefined enableStyleSheetsForSet (DOMString? name);
};

// https://drafts.csswg.org/cssom-view/#extensions-to-the-document-interface
partial interface Document {
    CaretPosition? caretPositionFromPoint (float x, float y);

    readonly attribute Element? scrollingElement;
};

// http://dev.w3.org/2006/webapi/selectors-api2/#interface-definitions
partial interface Document {
  [Throws, Pure]
  Element?  querySelector(DOMString selectors);
  [Throws, Pure]
  NodeList  querySelectorAll(DOMString selectors);

  //(Not implemented)Element?  find(DOMString selectors, optional (Element or sequence<Node>)? refNodes);
  //(Not implemented)NodeList  findAll(DOMString selectors, optional (Element or sequence<Node>)? refNodes);
};

// https://drafts.csswg.org/web-animations/#extensions-to-the-document-interface
partial interface Document {
  [Func="nsDocument::IsWebAnimationsEnabled"]
  readonly attribute DocumentTimeline timeline;
  [Func="nsDocument::IsWebAnimationsEnabled"]
  sequence<Animation> getAnimations();
};

// https://svgwg.org/svg2-draft/struct.html#InterfaceDocumentExtensions
partial interface Document {
  [BinaryName="SVGRootElement"]
  readonly attribute SVGSVGElement? rootElement;
};

dictionary BlockParsingOptions {
  /**
   * If true, blocks script-created parsers (created via document.open()) in
   * addition to network-created parsers.
   */
  boolean blockScriptCreated = true;
};

// Extension to give chrome JS the ability to determine when a document was
// created to satisfy an iframe with srcdoc attribute.
partial interface Document {
  [ChromeOnly] readonly attribute boolean isSrcdocDocument;
};


// Extension to give chrome JS the ability to get the underlying
// sandbox flag attribute
partial interface Document {
  [ChromeOnly] readonly attribute DOMString? sandboxFlagsAsString;
};


/**
 * Chrome document anonymous content management.
 * This is a Chrome-only API that allows inserting fixed positioned anonymous
 * content on top of the current page displayed in the document.
 * The supplied content is cloned and inserted into the document's CanvasFrame.
 * Note that this only works for HTML documents.
 */
partial interface Document {
  /**
   * Deep-clones the provided element and inserts it into the CanvasFrame.
   * Returns an AnonymousContent instance that can be used to manipulate the
   * inserted element.
   */
  [ChromeOnly, NewObject, Throws]
  AnonymousContent insertAnonymousContent(Element aElement);

  /**
   * Removes the element inserted into the CanvasFrame given an AnonymousContent
   * instance.
   */
  [ChromeOnly, Throws]
  undefined removeAnonymousContent(AnonymousContent aContent);
};

// http://w3c.github.io/selection-api/#extensions-to-document-interface
partial interface Document {
  [Throws]
  Selection? getSelection();
};

// Extension to give chrome JS the ability to determine whether
// the user has interacted with the document or not.
partial interface Document {
  [ChromeOnly] readonly attribute boolean userHasInteracted;
};

// Extension to give chrome JS the ability to simulate activate the docuement
// by user gesture.
partial interface Document {
  [ChromeOnly]
  undefined notifyUserGestureActivation();
};

// For more information on Flash classification, see
// toolkit/components/url-classifier/flash-block-lists.rst
enum FlashClassification {
  "unclassified",   // Denotes a classification that has not yet been computed.
                    // Allows for lazy classification.
  "unknown",        // Site is not on the whitelist or blacklist
  "allowed",        // Site is on the Flash whitelist
  "denied"          // Site is on the Flash blacklist
};
partial interface Document {
  [ChromeOnly]
  readonly attribute FlashClassification documentFlashClassification;
};

Document includes XPathEvaluator;
Document includes GlobalEventHandlers;
Document includes DocumentAndElementEventHandlers;
Document includes TouchEventHandlers;
Document includes ParentNode;
Document includes OnErrorEventHandlerForNodes;
Document includes GeometryUtils;
Document includes FontFaceSource;
Document includes DocumentOrShadowRoot;
