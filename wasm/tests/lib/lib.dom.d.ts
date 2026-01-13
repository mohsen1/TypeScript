// DOM lib.d.ts declarations
// These provide type definitions for browser DOM APIs

// Event handling
interface EventListener {
    (evt: Event): void;
}

interface EventListenerObject {
    handleEvent(object: Event): void;
}

type EventListenerOrEventListenerObject = EventListener | EventListenerObject;

interface AddEventListenerOptions extends EventListenerOptions {
    once?: boolean;
    passive?: boolean;
    signal?: AbortSignal;
}

interface EventListenerOptions {
    capture?: boolean;
}

// Event types
interface Event {
    readonly bubbles: boolean;
    cancelBubble: boolean;
    readonly cancelable: boolean;
    readonly composed: boolean;
    readonly currentTarget: EventTarget | null;
    readonly defaultPrevented: boolean;
    readonly eventPhase: number;
    readonly isTrusted: boolean;
    returnValue: boolean;
    readonly target: EventTarget | null;
    readonly timeStamp: DOMHighResTimeStamp;
    readonly type: string;
    composedPath(): EventTarget[];
    initEvent(type: string, bubbles?: boolean, cancelable?: boolean): void;
    preventDefault(): void;
    stopImmediatePropagation(): void;
    stopPropagation(): void;
    readonly AT_TARGET: 2;
    readonly BUBBLING_PHASE: 3;
    readonly CAPTURING_PHASE: 1;
    readonly NONE: 0;
}

interface EventInit {
    bubbles?: boolean;
    cancelable?: boolean;
    composed?: boolean;
}

declare var Event: {
    prototype: Event;
    new(type: string, eventInitDict?: EventInit): Event;
    readonly AT_TARGET: 2;
    readonly BUBBLING_PHASE: 3;
    readonly CAPTURING_PHASE: 1;
    readonly NONE: 0;
};

interface MouseEvent extends UIEvent {
    readonly altKey: boolean;
    readonly button: number;
    readonly buttons: number;
    readonly clientX: number;
    readonly clientY: number;
    readonly ctrlKey: boolean;
    readonly metaKey: boolean;
    readonly movementX: number;
    readonly movementY: number;
    readonly offsetX: number;
    readonly offsetY: number;
    readonly pageX: number;
    readonly pageY: number;
    readonly relatedTarget: EventTarget | null;
    readonly screenX: number;
    readonly screenY: number;
    readonly shiftKey: boolean;
    readonly x: number;
    readonly y: number;
    getModifierState(keyArg: string): boolean;
}

declare var MouseEvent: {
    prototype: MouseEvent;
    new(type: string, eventInitDict?: MouseEventInit): MouseEvent;
};

interface MouseEventInit extends EventModifierInit {
    button?: number;
    buttons?: number;
    clientX?: number;
    clientY?: number;
    movementX?: number;
    movementY?: number;
    relatedTarget?: EventTarget | null;
    screenX?: number;
    screenY?: number;
}

interface EventModifierInit extends UIEventInit {
    altKey?: boolean;
    ctrlKey?: boolean;
    metaKey?: boolean;
    modifierAltGraph?: boolean;
    modifierCapsLock?: boolean;
    modifierFn?: boolean;
    modifierFnLock?: boolean;
    modifierHyper?: boolean;
    modifierNumLock?: boolean;
    modifierScrollLock?: boolean;
    modifierSuper?: boolean;
    modifierSymbol?: boolean;
    modifierSymbolLock?: boolean;
    shiftKey?: boolean;
}

interface KeyboardEvent extends UIEvent {
    readonly altKey: boolean;
    readonly code: string;
    readonly ctrlKey: boolean;
    readonly isComposing: boolean;
    readonly key: string;
    readonly location: number;
    readonly metaKey: boolean;
    readonly repeat: boolean;
    readonly shiftKey: boolean;
    getModifierState(keyArg: string): boolean;
}

declare var KeyboardEvent: {
    prototype: KeyboardEvent;
    new(type: string, eventInitDict?: KeyboardEventInit): KeyboardEvent;
    readonly DOM_KEY_LOCATION_LEFT: 1;
    readonly DOM_KEY_LOCATION_NUMPAD: 3;
    readonly DOM_KEY_LOCATION_RIGHT: 2;
    readonly DOM_KEY_LOCATION_STANDARD: 0;
};

interface KeyboardEventInit extends EventModifierInit {
    code?: string;
    isComposing?: boolean;
    key?: string;
    location?: number;
    repeat?: boolean;
}

interface UIEvent extends Event {
    readonly detail: number;
    readonly view: Window | null;
}

interface UIEventInit extends EventInit {
    detail?: number;
    view?: Window | null;
}

declare var UIEvent: {
    prototype: UIEvent;
    new(type: string, eventInitDict?: UIEventInit): UIEvent;
};

interface FocusEvent extends UIEvent {
    readonly relatedTarget: EventTarget | null;
}

declare var FocusEvent: {
    prototype: FocusEvent;
    new(type: string, eventInitDict?: FocusEventInit): FocusEvent;
};

interface FocusEventInit extends UIEventInit {
    relatedTarget?: EventTarget | null;
}

interface InputEvent extends UIEvent {
    readonly data: string | null;
    readonly dataTransfer: DataTransfer | null;
    readonly inputType: string;
    readonly isComposing: boolean;
    getTargetRanges(): StaticRange[];
}

declare var InputEvent: {
    prototype: InputEvent;
    new(type: string, eventInitDict?: InputEventInit): InputEvent;
};

interface InputEventInit extends UIEventInit {
    data?: string | null;
    dataTransfer?: DataTransfer | null;
    inputType?: string;
    isComposing?: boolean;
    targetRanges?: StaticRange[];
}

// EventTarget
interface EventTarget {
    addEventListener(type: string, callback: EventListenerOrEventListenerObject | null, options?: AddEventListenerOptions | boolean): void;
    dispatchEvent(event: Event): boolean;
    removeEventListener(type: string, callback: EventListenerOrEventListenerObject | null, options?: EventListenerOptions | boolean): void;
}

declare var EventTarget: {
    prototype: EventTarget;
    new(): EventTarget;
};

// Abort Controller
interface AbortController {
    readonly signal: AbortSignal;
    abort(reason?: any): void;
}

declare var AbortController: {
    prototype: AbortController;
    new(): AbortController;
};

interface AbortSignal extends EventTarget {
    readonly aborted: boolean;
    readonly reason: any;
    onabort: ((this: AbortSignal, ev: Event) => any) | null;
    throwIfAborted(): void;
}

declare var AbortSignal: {
    prototype: AbortSignal;
    abort(reason?: any): AbortSignal;
    any(signals: AbortSignal[]): AbortSignal;
    timeout(milliseconds: number): AbortSignal;
};

// Node types
interface Node extends EventTarget {
    readonly baseURI: string;
    readonly childNodes: NodeListOf<ChildNode>;
    readonly firstChild: ChildNode | null;
    readonly isConnected: boolean;
    readonly lastChild: ChildNode | null;
    readonly nextSibling: ChildNode | null;
    readonly nodeName: string;
    readonly nodeType: number;
    nodeValue: string | null;
    readonly ownerDocument: Document | null;
    readonly parentElement: HTMLElement | null;
    readonly parentNode: ParentNode | null;
    readonly previousSibling: ChildNode | null;
    textContent: string | null;
    appendChild<T extends Node>(node: T): T;
    cloneNode(deep?: boolean): Node;
    compareDocumentPosition(other: Node): number;
    contains(other: Node | null): boolean;
    getRootNode(options?: GetRootNodeOptions): Node;
    hasChildNodes(): boolean;
    insertBefore<T extends Node>(node: T, child: Node | null): T;
    isDefaultNamespace(namespace: string | null): boolean;
    isEqualNode(otherNode: Node | null): boolean;
    isSameNode(otherNode: Node | null): boolean;
    lookupNamespaceURI(prefix: string | null): string | null;
    lookupPrefix(namespace: string | null): string | null;
    normalize(): void;
    removeChild<T extends Node>(child: T): T;
    replaceChild<T extends Node>(node: Node, child: T): T;
    readonly ATTRIBUTE_NODE: 2;
    readonly CDATA_SECTION_NODE: 4;
    readonly COMMENT_NODE: 8;
    readonly DOCUMENT_FRAGMENT_NODE: 11;
    readonly DOCUMENT_NODE: 9;
    readonly DOCUMENT_POSITION_CONTAINED_BY: 16;
    readonly DOCUMENT_POSITION_CONTAINS: 8;
    readonly DOCUMENT_POSITION_DISCONNECTED: 1;
    readonly DOCUMENT_POSITION_FOLLOWING: 4;
    readonly DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC: 32;
    readonly DOCUMENT_POSITION_PRECEDING: 2;
    readonly DOCUMENT_TYPE_NODE: 10;
    readonly ELEMENT_NODE: 1;
    readonly ENTITY_NODE: 6;
    readonly ENTITY_REFERENCE_NODE: 5;
    readonly NOTATION_NODE: 12;
    readonly PROCESSING_INSTRUCTION_NODE: 7;
    readonly TEXT_NODE: 3;
}

declare var Node: {
    prototype: Node;
    new(): Node;
    readonly ATTRIBUTE_NODE: 2;
    readonly CDATA_SECTION_NODE: 4;
    readonly COMMENT_NODE: 8;
    readonly DOCUMENT_FRAGMENT_NODE: 11;
    readonly DOCUMENT_NODE: 9;
    readonly DOCUMENT_POSITION_CONTAINED_BY: 16;
    readonly DOCUMENT_POSITION_CONTAINS: 8;
    readonly DOCUMENT_POSITION_DISCONNECTED: 1;
    readonly DOCUMENT_POSITION_FOLLOWING: 4;
    readonly DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC: 32;
    readonly DOCUMENT_POSITION_PRECEDING: 2;
    readonly DOCUMENT_TYPE_NODE: 10;
    readonly ELEMENT_NODE: 1;
    readonly ENTITY_NODE: 6;
    readonly ENTITY_REFERENCE_NODE: 5;
    readonly NOTATION_NODE: 12;
    readonly PROCESSING_INSTRUCTION_NODE: 7;
    readonly TEXT_NODE: 3;
};

interface GetRootNodeOptions {
    composed?: boolean;
}

interface ChildNode extends Node {
    after(...nodes: (Node | string)[]): void;
    before(...nodes: (Node | string)[]): void;
    remove(): void;
    replaceWith(...nodes: (Node | string)[]): void;
}

interface ParentNode extends Node {
    readonly childElementCount: number;
    readonly children: HTMLCollection;
    readonly firstElementChild: Element | null;
    readonly lastElementChild: Element | null;
    append(...nodes: (Node | string)[]): void;
    prepend(...nodes: (Node | string)[]): void;
    querySelector<K extends keyof HTMLElementTagNameMap>(selectors: K): HTMLElementTagNameMap[K] | null;
    querySelector(selectors: string): Element | null;
    querySelectorAll<K extends keyof HTMLElementTagNameMap>(selectors: K): NodeListOf<HTMLElementTagNameMap[K]>;
    querySelectorAll(selectors: string): NodeListOf<Element>;
    replaceChildren(...nodes: (Node | string)[]): void;
}

// Element
interface Element extends Node, ParentNode, ChildNode {
    readonly attributes: NamedNodeMap;
    readonly classList: DOMTokenList;
    className: string;
    readonly clientHeight: number;
    readonly clientLeft: number;
    readonly clientTop: number;
    readonly clientWidth: number;
    id: string;
    innerHTML: string;
    readonly localName: string;
    readonly namespaceURI: string | null;
    outerHTML: string;
    readonly prefix: string | null;
    readonly scrollHeight: number;
    scrollLeft: number;
    scrollTop: number;
    readonly scrollWidth: number;
    slot: string;
    readonly tagName: string;
    attachShadow(init: ShadowRootInit): ShadowRoot;
    closest<K extends keyof HTMLElementTagNameMap>(selector: K): HTMLElementTagNameMap[K] | null;
    closest(selectors: string): Element | null;
    getAttribute(qualifiedName: string): string | null;
    getAttributeNS(namespace: string | null, localName: string): string | null;
    getAttributeNames(): string[];
    getAttributeNode(qualifiedName: string): Attr | null;
    getAttributeNodeNS(namespace: string | null, localName: string): Attr | null;
    getBoundingClientRect(): DOMRect;
    getClientRects(): DOMRectList;
    getElementsByClassName(classNames: string): HTMLCollectionOf<Element>;
    getElementsByTagName<K extends keyof HTMLElementTagNameMap>(qualifiedName: K): HTMLCollectionOf<HTMLElementTagNameMap[K]>;
    getElementsByTagName(qualifiedName: string): HTMLCollectionOf<Element>;
    getElementsByTagNameNS(namespaceURI: string, localName: string): HTMLCollectionOf<Element>;
    hasAttribute(qualifiedName: string): boolean;
    hasAttributeNS(namespace: string | null, localName: string): boolean;
    hasAttributes(): boolean;
    insertAdjacentElement(where: InsertPosition, element: Element): Element | null;
    insertAdjacentHTML(position: InsertPosition, text: string): void;
    insertAdjacentText(where: InsertPosition, data: string): void;
    matches(selectors: string): boolean;
    removeAttribute(qualifiedName: string): void;
    removeAttributeNS(namespace: string | null, localName: string): void;
    removeAttributeNode(attr: Attr): Attr;
    requestFullscreen(options?: FullscreenOptions): Promise<void>;
    requestPointerLock(): void;
    scroll(options?: ScrollToOptions): void;
    scroll(x: number, y: number): void;
    scrollBy(options?: ScrollToOptions): void;
    scrollBy(x: number, y: number): void;
    scrollIntoView(arg?: boolean | ScrollIntoViewOptions): void;
    scrollTo(options?: ScrollToOptions): void;
    scrollTo(x: number, y: number): void;
    setAttribute(qualifiedName: string, value: string): void;
    setAttributeNS(namespace: string | null, qualifiedName: string, value: string): void;
    setAttributeNode(attr: Attr): Attr | null;
    setAttributeNodeNS(attr: Attr): Attr | null;
    toggleAttribute(qualifiedName: string, force?: boolean): boolean;
}

declare var Element: {
    prototype: Element;
    new(): Element;
};

type InsertPosition = "beforebegin" | "afterbegin" | "beforeend" | "afterend";

interface ScrollIntoViewOptions extends ScrollOptions {
    block?: ScrollLogicalPosition;
    inline?: ScrollLogicalPosition;
}

interface ScrollOptions {
    behavior?: ScrollBehavior;
}

interface ScrollToOptions extends ScrollOptions {
    left?: number;
    top?: number;
}

type ScrollBehavior = "auto" | "instant" | "smooth";
type ScrollLogicalPosition = "center" | "end" | "nearest" | "start";

interface FullscreenOptions {
    navigationUI?: FullscreenNavigationUI;
}

type FullscreenNavigationUI = "auto" | "hide" | "show";

// HTMLElement
interface HTMLElement extends Element {
    accessKey: string;
    readonly accessKeyLabel: string;
    autocapitalize: string;
    contentEditable: string;
    readonly dataset: DOMStringMap;
    dir: string;
    draggable: boolean;
    enterKeyHint: string;
    hidden: boolean;
    inert: boolean;
    innerText: string;
    inputMode: string;
    readonly isContentEditable: boolean;
    lang: string;
    nonce?: string;
    readonly offsetHeight: number;
    readonly offsetLeft: number;
    readonly offsetParent: Element | null;
    readonly offsetTop: number;
    readonly offsetWidth: number;
    outerText: string;
    popover: string | null;
    spellcheck: boolean;
    readonly style: CSSStyleDeclaration;
    tabIndex: number;
    title: string;
    translate: boolean;
    attachInternals(): ElementInternals;
    blur(): void;
    click(): void;
    focus(options?: FocusOptions): void;
    hidePopover(): void;
    showPopover(): void;
    togglePopover(force?: boolean): boolean;
}

declare var HTMLElement: {
    prototype: HTMLElement;
    new(): HTMLElement;
};

interface FocusOptions {
    preventScroll?: boolean;
}

// Common HTML Elements
interface HTMLDivElement extends HTMLElement {}
declare var HTMLDivElement: {
    prototype: HTMLDivElement;
    new(): HTMLDivElement;
};

interface HTMLSpanElement extends HTMLElement {}
declare var HTMLSpanElement: {
    prototype: HTMLSpanElement;
    new(): HTMLSpanElement;
};

interface HTMLParagraphElement extends HTMLElement {}
declare var HTMLParagraphElement: {
    prototype: HTMLParagraphElement;
    new(): HTMLParagraphElement;
};

interface HTMLHeadingElement extends HTMLElement {}
declare var HTMLHeadingElement: {
    prototype: HTMLHeadingElement;
    new(): HTMLHeadingElement;
};

interface HTMLAnchorElement extends HTMLElement {
    download: string;
    hash: string;
    host: string;
    hostname: string;
    href: string;
    readonly origin: string;
    password: string;
    pathname: string;
    port: string;
    protocol: string;
    referrerPolicy: string;
    rel: string;
    readonly relList: DOMTokenList;
    search: string;
    target: string;
    text: string;
    type: string;
    username: string;
}

declare var HTMLAnchorElement: {
    prototype: HTMLAnchorElement;
    new(): HTMLAnchorElement;
};

interface HTMLImageElement extends HTMLElement {
    alt: string;
    readonly complete: boolean;
    crossOrigin: string | null;
    readonly currentSrc: string;
    decoding: "async" | "sync" | "auto";
    fetchPriority: string;
    height: number;
    isMap: boolean;
    loading: "eager" | "lazy";
    readonly naturalHeight: number;
    readonly naturalWidth: number;
    referrerPolicy: string;
    sizes: string;
    src: string;
    srcset: string;
    useMap: string;
    width: number;
    readonly x: number;
    readonly y: number;
    decode(): Promise<void>;
}

declare var HTMLImageElement: {
    prototype: HTMLImageElement;
    new(): HTMLImageElement;
};

interface HTMLInputElement extends HTMLElement {
    accept: string;
    alt: string;
    autocomplete: AutoFill;
    capture: string;
    checked: boolean;
    defaultChecked: boolean;
    defaultValue: string;
    dirName: string;
    disabled: boolean;
    files: FileList | null;
    readonly form: HTMLFormElement | null;
    formAction: string;
    formEnctype: string;
    formMethod: string;
    formNoValidate: boolean;
    formTarget: string;
    height: number;
    indeterminate: boolean;
    readonly labels: NodeListOf<HTMLLabelElement> | null;
    readonly list: HTMLDataListElement | null;
    max: string;
    maxLength: number;
    min: string;
    minLength: number;
    multiple: boolean;
    name: string;
    pattern: string;
    placeholder: string;
    readOnly: boolean;
    required: boolean;
    selectionDirection: "forward" | "backward" | "none" | null;
    selectionEnd: number | null;
    selectionStart: number | null;
    size: number;
    src: string;
    step: string;
    type: string;
    readonly validationMessage: string;
    readonly validity: ValidityState;
    value: string;
    valueAsDate: Date | null;
    valueAsNumber: number;
    readonly webkitEntries: readonly FileSystemEntry[];
    webkitdirectory: boolean;
    width: number;
    readonly willValidate: boolean;
    checkValidity(): boolean;
    reportValidity(): boolean;
    select(): void;
    setCustomValidity(error: string): void;
    setRangeText(replacement: string): void;
    setRangeText(replacement: string, start: number, end: number, selectionMode?: SelectionMode): void;
    setSelectionRange(start: number | null, end: number | null, direction?: "forward" | "backward" | "none"): void;
    showPicker(): void;
    stepDown(n?: number): void;
    stepUp(n?: number): void;
}

declare var HTMLInputElement: {
    prototype: HTMLInputElement;
    new(): HTMLInputElement;
};

type AutoFill = string;
type SelectionMode = "select" | "start" | "end" | "preserve";

interface HTMLButtonElement extends HTMLElement {
    disabled: boolean;
    readonly form: HTMLFormElement | null;
    formAction: string;
    formEnctype: string;
    formMethod: string;
    formNoValidate: boolean;
    formTarget: string;
    readonly labels: NodeListOf<HTMLLabelElement>;
    name: string;
    type: "submit" | "reset" | "button";
    readonly validationMessage: string;
    readonly validity: ValidityState;
    value: string;
    readonly willValidate: boolean;
    checkValidity(): boolean;
    reportValidity(): boolean;
    setCustomValidity(error: string): void;
}

declare var HTMLButtonElement: {
    prototype: HTMLButtonElement;
    new(): HTMLButtonElement;
};

interface HTMLFormElement extends HTMLElement {
    acceptCharset: string;
    action: string;
    autocomplete: AutoFill;
    readonly elements: HTMLFormControlsCollection;
    encoding: string;
    enctype: string;
    readonly length: number;
    method: string;
    name: string;
    noValidate: boolean;
    target: string;
    checkValidity(): boolean;
    reportValidity(): boolean;
    requestSubmit(submitter?: HTMLElement | null): void;
    reset(): void;
    submit(): void;
    [index: number]: Element;
    [name: string]: any;
}

declare var HTMLFormElement: {
    prototype: HTMLFormElement;
    new(): HTMLFormElement;
};

interface HTMLTextAreaElement extends HTMLElement {
    autocomplete: AutoFill;
    cols: number;
    defaultValue: string;
    dirName: string;
    disabled: boolean;
    readonly form: HTMLFormElement | null;
    readonly labels: NodeListOf<HTMLLabelElement>;
    maxLength: number;
    minLength: number;
    name: string;
    placeholder: string;
    readOnly: boolean;
    required: boolean;
    rows: number;
    selectionDirection: "forward" | "backward" | "none";
    selectionEnd: number;
    selectionStart: number;
    readonly textLength: number;
    readonly validationMessage: string;
    readonly validity: ValidityState;
    value: string;
    readonly willValidate: boolean;
    wrap: string;
    checkValidity(): boolean;
    reportValidity(): boolean;
    select(): void;
    setCustomValidity(error: string): void;
    setRangeText(replacement: string): void;
    setRangeText(replacement: string, start: number, end: number, selectionMode?: SelectionMode): void;
    setSelectionRange(start: number | null, end: number | null, direction?: "forward" | "backward" | "none"): void;
}

declare var HTMLTextAreaElement: {
    prototype: HTMLTextAreaElement;
    new(): HTMLTextAreaElement;
};

interface HTMLSelectElement extends HTMLElement {
    autocomplete: AutoFill;
    disabled: boolean;
    readonly form: HTMLFormElement | null;
    readonly labels: NodeListOf<HTMLLabelElement>;
    length: number;
    multiple: boolean;
    name: string;
    readonly options: HTMLOptionsCollection;
    required: boolean;
    selectedIndex: number;
    readonly selectedOptions: HTMLCollectionOf<HTMLOptionElement>;
    size: number;
    readonly type: "select-one" | "select-multiple";
    readonly validationMessage: string;
    readonly validity: ValidityState;
    value: string;
    readonly willValidate: boolean;
    add(element: HTMLOptionElement | HTMLOptGroupElement, before?: HTMLElement | number | null): void;
    checkValidity(): boolean;
    item(index: number): HTMLOptionElement | null;
    namedItem(name: string): HTMLOptionElement | null;
    remove(index?: number): void;
    reportValidity(): boolean;
    setCustomValidity(error: string): void;
    [name: number]: HTMLOptionElement;
}

declare var HTMLSelectElement: {
    prototype: HTMLSelectElement;
    new(): HTMLSelectElement;
};

interface HTMLOptionElement extends HTMLElement {
    defaultSelected: boolean;
    disabled: boolean;
    readonly form: HTMLFormElement | null;
    readonly index: number;
    label: string;
    selected: boolean;
    text: string;
    value: string;
}

declare var HTMLOptionElement: {
    prototype: HTMLOptionElement;
    new(): HTMLOptionElement;
};

interface HTMLLabelElement extends HTMLElement {
    readonly control: HTMLElement | null;
    readonly form: HTMLFormElement | null;
    htmlFor: string;
}

declare var HTMLLabelElement: {
    prototype: HTMLLabelElement;
    new(): HTMLLabelElement;
};

interface HTMLCanvasElement extends HTMLElement {
    height: number;
    width: number;
    getContext(contextId: "2d", options?: CanvasRenderingContext2DSettings): CanvasRenderingContext2D | null;
    getContext(contextId: "webgl", options?: WebGLContextAttributes): WebGLRenderingContext | null;
    getContext(contextId: "webgl2", options?: WebGLContextAttributes): WebGL2RenderingContext | null;
    getContext(contextId: string, options?: any): RenderingContext | null;
    toBlob(callback: BlobCallback, type?: string, quality?: number): void;
    toDataURL(type?: string, quality?: number): string;
    transferControlToOffscreen(): OffscreenCanvas;
}

declare var HTMLCanvasElement: {
    prototype: HTMLCanvasElement;
    new(): HTMLCanvasElement;
};

type BlobCallback = (blob: Blob | null) => void;
type RenderingContext = CanvasRenderingContext2D | WebGLRenderingContext | WebGL2RenderingContext;

interface HTMLScriptElement extends HTMLElement {
    async: boolean;
    crossOrigin: string | null;
    defer: boolean;
    integrity: string;
    noModule: boolean;
    referrerPolicy: string;
    src: string;
    text: string;
    type: string;
}

declare var HTMLScriptElement: {
    prototype: HTMLScriptElement;
    new(): HTMLScriptElement;
};

interface HTMLStyleElement extends HTMLElement {
    disabled: boolean;
    media: string;
    readonly sheet: CSSStyleSheet | null;
}

declare var HTMLStyleElement: {
    prototype: HTMLStyleElement;
    new(): HTMLStyleElement;
};

interface HTMLLinkElement extends HTMLElement {
    as: string;
    crossOrigin: string | null;
    disabled: boolean;
    href: string;
    hreflang: string;
    integrity: string;
    media: string;
    referrerPolicy: string;
    rel: string;
    readonly relList: DOMTokenList;
    readonly sheet: CSSStyleSheet | null;
    type: string;
}

declare var HTMLLinkElement: {
    prototype: HTMLLinkElement;
    new(): HTMLLinkElement;
};

// Document
interface Document extends Node, ParentNode {
    readonly URL: string;
    readonly activeElement: Element | null;
    readonly body: HTMLElement | null;
    readonly characterSet: string;
    readonly childElementCount: number;
    readonly children: HTMLCollection;
    readonly compatMode: string;
    readonly contentType: string;
    cookie: string;
    readonly currentScript: HTMLOrSVGScriptElement | null;
    readonly defaultView: Window | null;
    designMode: string;
    dir: string;
    readonly doctype: DocumentType | null;
    readonly documentElement: HTMLElement;
    readonly documentURI: string;
    domain: string;
    readonly firstElementChild: Element | null;
    readonly forms: HTMLCollectionOf<HTMLFormElement>;
    readonly fullscreenElement: Element | null;
    readonly head: HTMLHeadElement;
    readonly hidden: boolean;
    readonly images: HTMLCollectionOf<HTMLImageElement>;
    readonly lastElementChild: Element | null;
    readonly lastModified: string;
    readonly links: HTMLCollectionOf<HTMLAnchorElement | HTMLAreaElement>;
    readonly location: Location;
    readonly readyState: DocumentReadyState;
    readonly referrer: string;
    readonly scripts: HTMLCollectionOf<HTMLScriptElement>;
    readonly scrollingElement: Element | null;
    readonly timeline: DocumentTimeline;
    title: string;
    readonly visibilityState: DocumentVisibilityState;
    adoptNode<T extends Node>(node: T): T;
    close(): void;
    createAttribute(localName: string): Attr;
    createAttributeNS(namespace: string | null, qualifiedName: string): Attr;
    createComment(data: string): Comment;
    createDocumentFragment(): DocumentFragment;
    createElement<K extends keyof HTMLElementTagNameMap>(tagName: K, options?: ElementCreationOptions): HTMLElementTagNameMap[K];
    createElement(tagName: string, options?: ElementCreationOptions): HTMLElement;
    createElementNS(namespaceURI: string | null, qualifiedName: string, options?: ElementCreationOptions): Element;
    createEvent(eventInterface: string): Event;
    createNodeIterator(root: Node, whatToShow?: number, filter?: NodeFilter | null): NodeIterator;
    createProcessingInstruction(target: string, data: string): ProcessingInstruction;
    createRange(): Range;
    createTextNode(data: string): Text;
    createTreeWalker(root: Node, whatToShow?: number, filter?: NodeFilter | null): TreeWalker;
    execCommand(commandId: string, showUI?: boolean, value?: string): boolean;
    exitFullscreen(): Promise<void>;
    exitPointerLock(): void;
    getElementById(elementId: string): HTMLElement | null;
    getElementsByClassName(classNames: string): HTMLCollectionOf<Element>;
    getElementsByName(elementName: string): NodeListOf<HTMLElement>;
    getElementsByTagName<K extends keyof HTMLElementTagNameMap>(qualifiedName: K): HTMLCollectionOf<HTMLElementTagNameMap[K]>;
    getElementsByTagName(qualifiedName: string): HTMLCollectionOf<Element>;
    getElementsByTagNameNS(namespaceURI: string | null, localName: string): HTMLCollectionOf<Element>;
    hasFocus(): boolean;
    importNode<T extends Node>(node: T, deep?: boolean): T;
    open(unused1?: string, unused2?: string): Document;
    open(url: string | URL, name: string, features: string): Window | null;
    queryCommandEnabled(commandId: string): boolean;
    queryCommandState(commandId: string): boolean;
    queryCommandSupported(commandId: string): boolean;
    queryCommandValue(commandId: string): string;
    write(...text: string[]): void;
    writeln(...text: string[]): void;
}

declare var Document: {
    prototype: Document;
    new(): Document;
};

type DocumentReadyState = "loading" | "interactive" | "complete";
type DocumentVisibilityState = "hidden" | "visible";

interface ElementCreationOptions {
    is?: string;
}

// Window
interface Window extends EventTarget {
    readonly closed: boolean;
    readonly devicePixelRatio: number;
    readonly document: Document;
    readonly frameElement: Element | null;
    readonly frames: Window;
    readonly history: History;
    readonly innerHeight: number;
    readonly innerWidth: number;
    readonly length: number;
    readonly location: Location;
    name: string;
    readonly navigator: Navigator;
    opener: any;
    readonly outerHeight: number;
    readonly outerWidth: number;
    readonly pageXOffset: number;
    readonly pageYOffset: number;
    readonly parent: Window;
    readonly performance: Performance;
    readonly screen: Screen;
    readonly screenLeft: number;
    readonly screenTop: number;
    readonly screenX: number;
    readonly screenY: number;
    readonly scrollX: number;
    readonly scrollY: number;
    readonly self: Window;
    readonly sessionStorage: Storage;
    readonly localStorage: Storage;
    readonly speechSynthesis: SpeechSynthesis;
    status: string;
    readonly top: Window | null;
    readonly visualViewport: VisualViewport | null;
    readonly window: Window;
    alert(message?: any): void;
    blur(): void;
    cancelAnimationFrame(handle: number): void;
    cancelIdleCallback(handle: number): void;
    close(): void;
    confirm(message?: string): boolean;
    focus(): void;
    getComputedStyle(elt: Element, pseudoElt?: string | null): CSSStyleDeclaration;
    getSelection(): Selection | null;
    matchMedia(query: string): MediaQueryList;
    moveBy(x: number, y: number): void;
    moveTo(x: number, y: number): void;
    open(url?: string | URL, target?: string, features?: string): Window | null;
    postMessage(message: any, targetOrigin: string, transfer?: Transferable[]): void;
    postMessage(message: any, options?: WindowPostMessageOptions): void;
    print(): void;
    prompt(message?: string, _default?: string): string | null;
    requestAnimationFrame(callback: FrameRequestCallback): number;
    requestIdleCallback(callback: IdleRequestCallback, options?: IdleRequestOptions): number;
    resizeBy(x: number, y: number): void;
    resizeTo(width: number, height: number): void;
    scroll(options?: ScrollToOptions): void;
    scroll(x: number, y: number): void;
    scrollBy(options?: ScrollToOptions): void;
    scrollBy(x: number, y: number): void;
    scrollTo(options?: ScrollToOptions): void;
    scrollTo(x: number, y: number): void;
    stop(): void;
    atob(data: string): string;
    btoa(data: string): string;
    clearInterval(id?: number): void;
    clearTimeout(id?: number): void;
    createImageBitmap(image: ImageBitmapSource, options?: ImageBitmapOptions): Promise<ImageBitmap>;
    createImageBitmap(image: ImageBitmapSource, sx: number, sy: number, sw: number, sh: number, options?: ImageBitmapOptions): Promise<ImageBitmap>;
    fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response>;
    queueMicrotask(callback: VoidFunction): void;
    reportError(e: any): void;
    setInterval(handler: TimerHandler, timeout?: number, ...arguments: any[]): number;
    setTimeout(handler: TimerHandler, timeout?: number, ...arguments: any[]): number;
    structuredClone<T>(value: T, options?: StructuredSerializeOptions): T;
}

declare var Window: {
    prototype: Window;
    new(): Window;
};

declare var window: Window;
declare var document: Document;

type FrameRequestCallback = (time: DOMHighResTimeStamp) => void;
type IdleRequestCallback = (deadline: IdleDeadline) => void;
type TimerHandler = string | Function;
type VoidFunction = () => void;
type DOMHighResTimeStamp = number;

interface IdleRequestOptions {
    timeout?: number;
}

interface IdleDeadline {
    readonly didTimeout: boolean;
    timeRemaining(): DOMHighResTimeStamp;
}

interface WindowPostMessageOptions extends StructuredSerializeOptions {
    targetOrigin?: string;
}

interface StructuredSerializeOptions {
    transfer?: Transferable[];
}

// Location and History
interface Location {
    readonly ancestorOrigins: DOMStringList;
    hash: string;
    host: string;
    hostname: string;
    href: string;
    readonly origin: string;
    pathname: string;
    port: string;
    protocol: string;
    search: string;
    assign(url: string | URL): void;
    reload(): void;
    replace(url: string | URL): void;
    toString(): string;
}

declare var Location: {
    prototype: Location;
    new(): Location;
};

interface History {
    readonly length: number;
    scrollRestoration: ScrollRestoration;
    readonly state: any;
    back(): void;
    forward(): void;
    go(delta?: number): void;
    pushState(data: any, unused: string, url?: string | URL | null): void;
    replaceState(data: any, unused: string, url?: string | URL | null): void;
}

declare var History: {
    prototype: History;
    new(): History;
};

type ScrollRestoration = "auto" | "manual";

// Navigator
interface Navigator {
    readonly clipboard: Clipboard;
    readonly cookieEnabled: boolean;
    readonly geolocation: Geolocation;
    readonly hardwareConcurrency: number;
    readonly language: string;
    readonly languages: readonly string[];
    readonly maxTouchPoints: number;
    readonly mediaDevices: MediaDevices;
    readonly onLine: boolean;
    readonly pdfViewerEnabled: boolean;
    readonly permissions: Permissions;
    readonly platform: string;
    readonly serviceWorker: ServiceWorkerContainer;
    readonly userAgent: string;
    readonly webdriver: boolean;
    canShare(data?: ShareData): boolean;
    getGamepads(): (Gamepad | null)[];
    sendBeacon(url: string | URL, data?: BodyInit | null): boolean;
    share(data?: ShareData): Promise<void>;
    vibrate(pattern: VibratePattern): boolean;
}

declare var Navigator: {
    prototype: Navigator;
    new(): Navigator;
};

declare var navigator: Navigator;

type VibratePattern = number | number[];

interface ShareData {
    files?: File[];
    text?: string;
    title?: string;
    url?: string;
}

// Storage
interface Storage {
    readonly length: number;
    clear(): void;
    getItem(key: string): string | null;
    key(index: number): string | null;
    removeItem(key: string): void;
    setItem(key: string, value: string): void;
    [name: string]: any;
}

declare var Storage: {
    prototype: Storage;
    new(): Storage;
};

declare var localStorage: Storage;
declare var sessionStorage: Storage;

// Fetch API
interface Body {
    readonly body: ReadableStream<Uint8Array> | null;
    readonly bodyUsed: boolean;
    arrayBuffer(): Promise<ArrayBuffer>;
    blob(): Promise<Blob>;
    formData(): Promise<FormData>;
    json(): Promise<any>;
    text(): Promise<string>;
}

interface Request extends Body {
    readonly cache: RequestCache;
    readonly credentials: RequestCredentials;
    readonly destination: RequestDestination;
    readonly headers: Headers;
    readonly integrity: string;
    readonly keepalive: boolean;
    readonly method: string;
    readonly mode: RequestMode;
    readonly redirect: RequestRedirect;
    readonly referrer: string;
    readonly referrerPolicy: ReferrerPolicy;
    readonly signal: AbortSignal;
    readonly url: string;
    clone(): Request;
}

declare var Request: {
    prototype: Request;
    new(input: RequestInfo | URL, init?: RequestInit): Request;
};

type RequestInfo = Request | string;
type RequestCache = "default" | "force-cache" | "no-cache" | "no-store" | "only-if-cached" | "reload";
type RequestCredentials = "include" | "omit" | "same-origin";
type RequestDestination = "" | "audio" | "audioworklet" | "document" | "embed" | "font" | "frame" | "iframe" | "image" | "manifest" | "object" | "paintworklet" | "report" | "script" | "sharedworker" | "style" | "track" | "video" | "worker" | "xslt";
type RequestMode = "cors" | "navigate" | "no-cors" | "same-origin";
type RequestRedirect = "error" | "follow" | "manual";
type ReferrerPolicy = "" | "no-referrer" | "no-referrer-when-downgrade" | "origin" | "origin-when-cross-origin" | "same-origin" | "strict-origin" | "strict-origin-when-cross-origin" | "unsafe-url";

interface RequestInit {
    body?: BodyInit | null;
    cache?: RequestCache;
    credentials?: RequestCredentials;
    headers?: HeadersInit;
    integrity?: string;
    keepalive?: boolean;
    method?: string;
    mode?: RequestMode;
    redirect?: RequestRedirect;
    referrer?: string;
    referrerPolicy?: ReferrerPolicy;
    signal?: AbortSignal | null;
    window?: null;
}

type BodyInit = ReadableStream<Uint8Array> | Blob | BufferSource | FormData | URLSearchParams | string;
type BufferSource = ArrayBufferView | ArrayBuffer;
type HeadersInit = [string, string][] | Record<string, string> | Headers;

interface Response extends Body {
    readonly headers: Headers;
    readonly ok: boolean;
    readonly redirected: boolean;
    readonly status: number;
    readonly statusText: string;
    readonly type: ResponseType;
    readonly url: string;
    clone(): Response;
}

declare var Response: {
    prototype: Response;
    new(body?: BodyInit | null, init?: ResponseInit): Response;
    error(): Response;
    json(data: any, init?: ResponseInit): Response;
    redirect(url: string | URL, status?: number): Response;
};

type ResponseType = "basic" | "cors" | "default" | "error" | "opaque" | "opaqueredirect";

interface ResponseInit {
    headers?: HeadersInit;
    status?: number;
    statusText?: string;
}

interface Headers {
    append(name: string, value: string): void;
    delete(name: string): void;
    get(name: string): string | null;
    has(name: string): boolean;
    set(name: string, value: string): void;
    forEach(callbackfn: (value: string, key: string, parent: Headers) => void): void;
    entries(): IterableIterator<[string, string]>;
    keys(): IterableIterator<string>;
    values(): IterableIterator<string>;
}

declare var Headers: {
    prototype: Headers;
    new(init?: HeadersInit): Headers;
};

declare function fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response>;

// URL API
interface URL {
    hash: string;
    host: string;
    hostname: string;
    href: string;
    readonly origin: string;
    password: string;
    pathname: string;
    port: string;
    protocol: string;
    search: string;
    readonly searchParams: URLSearchParams;
    username: string;
    toJSON(): string;
    toString(): string;
}

declare var URL: {
    prototype: URL;
    new(url: string | URL, base?: string | URL): URL;
    canParse(url: string | URL, base?: string | URL): boolean;
    createObjectURL(obj: Blob | MediaSource): string;
    revokeObjectURL(url: string): void;
};

interface URLSearchParams {
    append(name: string, value: string): void;
    delete(name: string, value?: string): void;
    get(name: string): string | null;
    getAll(name: string): string[];
    has(name: string, value?: string): boolean;
    set(name: string, value: string): void;
    sort(): void;
    toString(): string;
    forEach(callbackfn: (value: string, key: string, parent: URLSearchParams) => void): void;
    entries(): IterableIterator<[string, string]>;
    keys(): IterableIterator<string>;
    values(): IterableIterator<string>;
    readonly size: number;
}

declare var URLSearchParams: {
    prototype: URLSearchParams;
    new(init?: string[][] | Record<string, string> | string | URLSearchParams): URLSearchParams;
};

// Blob and File
interface Blob {
    readonly size: number;
    readonly type: string;
    arrayBuffer(): Promise<ArrayBuffer>;
    slice(start?: number, end?: number, contentType?: string): Blob;
    stream(): ReadableStream<Uint8Array>;
    text(): Promise<string>;
}

declare var Blob: {
    prototype: Blob;
    new(blobParts?: BlobPart[], options?: BlobPropertyBag): Blob;
};

type BlobPart = BufferSource | Blob | string;

interface BlobPropertyBag {
    endings?: EndingType;
    type?: string;
}

type EndingType = "native" | "transparent";

interface File extends Blob {
    readonly lastModified: number;
    readonly name: string;
    readonly webkitRelativePath: string;
}

declare var File: {
    prototype: File;
    new(fileBits: BlobPart[], fileName: string, options?: FilePropertyBag): File;
};

interface FilePropertyBag extends BlobPropertyBag {
    lastModified?: number;
}

interface FileList {
    readonly length: number;
    item(index: number): File | null;
    [index: number]: File;
}

declare var FileList: {
    prototype: FileList;
    new(): FileList;
};

interface FileReader extends EventTarget {
    readonly error: DOMException | null;
    onabort: ((this: FileReader, ev: ProgressEvent<FileReader>) => any) | null;
    onerror: ((this: FileReader, ev: ProgressEvent<FileReader>) => any) | null;
    onload: ((this: FileReader, ev: ProgressEvent<FileReader>) => any) | null;
    onloadend: ((this: FileReader, ev: ProgressEvent<FileReader>) => any) | null;
    onloadstart: ((this: FileReader, ev: ProgressEvent<FileReader>) => any) | null;
    onprogress: ((this: FileReader, ev: ProgressEvent<FileReader>) => any) | null;
    readonly readyState: typeof FileReader.DONE | typeof FileReader.EMPTY | typeof FileReader.LOADING;
    readonly result: string | ArrayBuffer | null;
    abort(): void;
    readAsArrayBuffer(blob: Blob): void;
    readAsBinaryString(blob: Blob): void;
    readAsDataURL(blob: Blob): void;
    readAsText(blob: Blob, encoding?: string): void;
    readonly DONE: 2;
    readonly EMPTY: 0;
    readonly LOADING: 1;
}

declare var FileReader: {
    prototype: FileReader;
    new(): FileReader;
    readonly DONE: 2;
    readonly EMPTY: 0;
    readonly LOADING: 1;
};

interface ProgressEvent<T extends EventTarget = EventTarget> extends Event {
    readonly lengthComputable: boolean;
    readonly loaded: number;
    readonly target: T | null;
    readonly total: number;
}

declare var ProgressEvent: {
    prototype: ProgressEvent;
    new(type: string, eventInitDict?: ProgressEventInit): ProgressEvent;
};

interface ProgressEventInit extends EventInit {
    lengthComputable?: boolean;
    loaded?: number;
    total?: number;
}

// FormData
interface FormData {
    append(name: string, value: string | Blob, fileName?: string): void;
    delete(name: string): void;
    get(name: string): FormDataEntryValue | null;
    getAll(name: string): FormDataEntryValue[];
    has(name: string): boolean;
    set(name: string, value: string | Blob, fileName?: string): void;
    forEach(callbackfn: (value: FormDataEntryValue, key: string, parent: FormData) => void): void;
    entries(): IterableIterator<[string, FormDataEntryValue]>;
    keys(): IterableIterator<string>;
    values(): IterableIterator<FormDataEntryValue>;
}

declare var FormData: {
    prototype: FormData;
    new(form?: HTMLFormElement, submitter?: HTMLElement | null): FormData;
};

type FormDataEntryValue = File | string;

// Collections
interface NodeList {
    readonly length: number;
    item(index: number): Node | null;
    forEach(callbackfn: (value: Node, key: number, parent: NodeList) => void): void;
    entries(): IterableIterator<[number, Node]>;
    keys(): IterableIterator<number>;
    values(): IterableIterator<Node>;
    [index: number]: Node;
}

declare var NodeList: {
    prototype: NodeList;
    new(): NodeList;
};

interface NodeListOf<TNode extends Node> extends NodeList {
    item(index: number): TNode;
    forEach(callbackfn: (value: TNode, key: number, parent: NodeListOf<TNode>) => void): void;
    entries(): IterableIterator<[number, TNode]>;
    keys(): IterableIterator<number>;
    values(): IterableIterator<TNode>;
    [index: number]: TNode;
}

interface HTMLCollection {
    readonly length: number;
    item(index: number): Element | null;
    namedItem(name: string): Element | null;
    [index: number]: Element;
}

declare var HTMLCollection: {
    prototype: HTMLCollection;
    new(): HTMLCollection;
};

interface HTMLCollectionOf<T extends Element> extends HTMLCollection {
    item(index: number): T | null;
    namedItem(name: string): T | null;
    [index: number]: T;
}

interface HTMLFormControlsCollection extends HTMLCollection {
    namedItem(name: string): RadioNodeList | Element | null;
}

declare var HTMLFormControlsCollection: {
    prototype: HTMLFormControlsCollection;
    new(): HTMLFormControlsCollection;
};

interface RadioNodeList extends NodeList {
    value: string;
}

declare var RadioNodeList: {
    prototype: RadioNodeList;
    new(): RadioNodeList;
};

interface HTMLOptionsCollection extends HTMLCollection {
    length: number;
    selectedIndex: number;
    add(element: HTMLOptionElement | HTMLOptGroupElement, before?: HTMLElement | number | null): void;
    remove(index: number): void;
}

declare var HTMLOptionsCollection: {
    prototype: HTMLOptionsCollection;
    new(): HTMLOptionsCollection;
};

interface NamedNodeMap {
    readonly length: number;
    getNamedItem(qualifiedName: string): Attr | null;
    getNamedItemNS(namespace: string | null, localName: string): Attr | null;
    item(index: number): Attr | null;
    removeNamedItem(qualifiedName: string): Attr;
    removeNamedItemNS(namespace: string | null, localName: string): Attr;
    setNamedItem(attr: Attr): Attr | null;
    setNamedItemNS(attr: Attr): Attr | null;
    [index: number]: Attr;
}

declare var NamedNodeMap: {
    prototype: NamedNodeMap;
    new(): NamedNodeMap;
};

interface DOMStringList {
    readonly length: number;
    contains(string: string): boolean;
    item(index: number): string | null;
    [index: number]: string;
}

declare var DOMStringList: {
    prototype: DOMStringList;
    new(): DOMStringList;
};

interface DOMStringMap {
    [name: string]: string | undefined;
}

declare var DOMStringMap: {
    prototype: DOMStringMap;
    new(): DOMStringMap;
};

interface DOMTokenList {
    readonly length: number;
    value: string;
    add(...tokens: string[]): void;
    contains(token: string): boolean;
    item(index: number): string | null;
    remove(...tokens: string[]): void;
    replace(token: string, newToken: string): boolean;
    supports(token: string): boolean;
    toggle(token: string, force?: boolean): boolean;
    forEach(callbackfn: (value: string, key: number, parent: DOMTokenList) => void): void;
    entries(): IterableIterator<[number, string]>;
    keys(): IterableIterator<number>;
    values(): IterableIterator<string>;
    [index: number]: string;
}

declare var DOMTokenList: {
    prototype: DOMTokenList;
    new(): DOMTokenList;
};

// DOMRect
interface DOMRect extends DOMRectReadOnly {
    height: number;
    width: number;
    x: number;
    y: number;
}

declare var DOMRect: {
    prototype: DOMRect;
    new(x?: number, y?: number, width?: number, height?: number): DOMRect;
    fromRect(other?: DOMRectInit): DOMRect;
};

interface DOMRectReadOnly {
    readonly bottom: number;
    readonly height: number;
    readonly left: number;
    readonly right: number;
    readonly top: number;
    readonly width: number;
    readonly x: number;
    readonly y: number;
    toJSON(): any;
}

declare var DOMRectReadOnly: {
    prototype: DOMRectReadOnly;
    new(x?: number, y?: number, width?: number, height?: number): DOMRectReadOnly;
    fromRect(other?: DOMRectInit): DOMRectReadOnly;
};

interface DOMRectInit {
    height?: number;
    width?: number;
    x?: number;
    y?: number;
}

interface DOMRectList {
    readonly length: number;
    item(index: number): DOMRect | null;
    [index: number]: DOMRect;
}

declare var DOMRectList: {
    prototype: DOMRectList;
    new(): DOMRectList;
};

// Attribute
interface Attr extends Node {
    readonly localName: string;
    readonly name: string;
    readonly namespaceURI: string | null;
    readonly ownerDocument: Document;
    readonly ownerElement: Element | null;
    readonly prefix: string | null;
    readonly specified: boolean;
    value: string;
}

declare var Attr: {
    prototype: Attr;
    new(): Attr;
};

// Text and Comment nodes
interface CharacterData extends Node, ChildNode {
    data: string;
    readonly length: number;
    appendData(data: string): void;
    deleteData(offset: number, count: number): void;
    insertData(offset: number, data: string): void;
    replaceData(offset: number, count: number, data: string): void;
    substringData(offset: number, count: number): string;
}

declare var CharacterData: {
    prototype: CharacterData;
    new(): CharacterData;
};

interface Text extends CharacterData {
    readonly assignedSlot: HTMLSlotElement | null;
    readonly wholeText: string;
    splitText(offset: number): Text;
}

declare var Text: {
    prototype: Text;
    new(data?: string): Text;
};

interface Comment extends CharacterData {
}

declare var Comment: {
    prototype: Comment;
    new(data?: string): Comment;
};

// HTMLElementTagNameMap for better querySelector/createElement typing
interface HTMLElementTagNameMap {
    "a": HTMLAnchorElement;
    "button": HTMLButtonElement;
    "canvas": HTMLCanvasElement;
    "div": HTMLDivElement;
    "form": HTMLFormElement;
    "h1": HTMLHeadingElement;
    "h2": HTMLHeadingElement;
    "h3": HTMLHeadingElement;
    "h4": HTMLHeadingElement;
    "h5": HTMLHeadingElement;
    "h6": HTMLHeadingElement;
    "img": HTMLImageElement;
    "input": HTMLInputElement;
    "label": HTMLLabelElement;
    "link": HTMLLinkElement;
    "option": HTMLOptionElement;
    "p": HTMLParagraphElement;
    "script": HTMLScriptElement;
    "select": HTMLSelectElement;
    "span": HTMLSpanElement;
    "style": HTMLStyleElement;
    "textarea": HTMLTextAreaElement;
}

// Additional placeholder interfaces (empty for now, can be expanded)
interface CSSStyleDeclaration {
    cssText: string;
    readonly length: number;
    readonly parentRule: CSSRule | null;
    getPropertyPriority(property: string): string;
    getPropertyValue(property: string): string;
    item(index: number): string;
    removeProperty(property: string): string;
    setProperty(property: string, value: string | null, priority?: string): void;
    [index: number]: string;
    [property: string]: any;
}

declare var CSSStyleDeclaration: {
    prototype: CSSStyleDeclaration;
    new(): CSSStyleDeclaration;
};

interface CSSStyleSheet extends StyleSheet {}
interface CSSRule {}
interface StyleSheet {}
interface DocumentFragment extends Node, ParentNode {}
declare var DocumentFragment: { prototype: DocumentFragment; new(): DocumentFragment; };
interface DocumentType extends Node {}
interface HTMLHeadElement extends HTMLElement {}
interface HTMLAreaElement extends HTMLElement {}
interface HTMLOrSVGScriptElement {}
interface HTMLDataListElement extends HTMLElement {}
interface HTMLOptGroupElement extends HTMLElement {}
interface HTMLSlotElement extends HTMLElement {}
interface ShadowRoot extends Node {}
interface ShadowRootInit {}
interface DataTransfer {}
interface StaticRange {}
interface Range {}
interface Selection {}
interface NodeIterator {}
interface TreeWalker {}
interface ProcessingInstruction extends Node {}
interface NodeFilter {}
interface DocumentTimeline {}
interface ValidityState {}
interface ElementInternals {}
interface FileSystemEntry {}
interface MediaQueryList {}
interface Screen {}
interface Performance {}
interface VisualViewport {}
interface SpeechSynthesis {}
interface Clipboard {}
interface Geolocation {}
interface MediaDevices {}
interface Permissions {}
interface ServiceWorkerContainer {}
interface Gamepad {}
interface CanvasRenderingContext2DSettings {}
interface CanvasRenderingContext2D {}
interface WebGLContextAttributes {}
interface WebGLRenderingContext {}
interface WebGL2RenderingContext {}
interface OffscreenCanvas {}
interface ImageBitmap {}
interface ImageBitmapOptions {}
type ImageBitmapSource = Blob | ImageData | HTMLImageElement | HTMLCanvasElement | HTMLVideoElement | OffscreenCanvas | ImageBitmap;
interface ImageData {}
interface HTMLVideoElement extends HTMLElement {}
interface MediaSource {}
interface ReadableStream<R = any> {}
type Transferable = ArrayBuffer | MessagePort | ImageBitmap | OffscreenCanvas;
interface MessagePort {}
