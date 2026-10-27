export function el<Tag extends keyof HTMLElementTagNameMap>(
  tag: Tag,
  properties: Partial<HTMLElementTagNameMap[Tag]> = {},
  ...children: (Node | string)[]
): HTMLElementTagNameMap[Tag] {
  const element = Object.assign(document.createElement(tag), properties);
  element.append(...children);
  return element;
}
