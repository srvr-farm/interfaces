#!/bin/sh
set -eu

: "${BIN:?BIN is required}"
: "${VERSION:?VERSION is required}"

CAPABILITY="${CAPABILITY:-}"
PACKAGE_OUTPUT_DIR="${PACKAGE_OUTPUT_DIR:-dist}"
DEB_ARCH="${DEB_ARCH:-amd64}"
RPM_ARCH="${RPM_ARCH:-x86_64}"
RPM_RELEASE="${RPM_RELEASE:-1}"

deb="${PACKAGE_OUTPUT_DIR}/${BIN}_${VERSION}_${DEB_ARCH}.deb"
rpm="${PACKAGE_OUTPUT_DIR}/${BIN}-${VERSION}-${RPM_RELEASE}.${RPM_ARCH}.rpm"

for artifact in "$deb" "$deb.sha256" "$rpm" "$rpm.sha256"; do
  test -s "$artifact" || {
    echo "missing package artifact: $artifact" >&2
    exit 1
  }
done

sha256sum -c "$deb.sha256"
sha256sum -c "$rpm.sha256"

test "$(dpkg-deb -f "$deb" Package)" = "$BIN"
test "$(dpkg-deb -f "$deb" Version)" = "$VERSION"
test "$(dpkg-deb -f "$deb" Architecture)" = "$DEB_ARCH"
deb_depends="$(dpkg-deb -f "$deb" Depends 2>/dev/null || true)"
if [ -n "$CAPABILITY" ]; then
  printf '%s' "$deb_depends" | grep -F "libcap2-bin" >/dev/null
else
  if printf '%s' "$deb_depends" | grep -F "libcap2-bin" >/dev/null; then
    echo "unexpected libcap2-bin dependency for package without capabilities" >&2
    exit 1
  fi
fi
dpkg-deb -c "$deb" | grep -Eq "^-rwxr-xr-x +root/root +[0-9]+ .* \\./usr/bin/${BIN}$"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
dpkg-deb --control "$deb" "$tmp/deb-control"
grep -F "chmod 0755 /usr/bin/${BIN}" "$tmp/deb-control/postinst" >/dev/null
if [ -n "$CAPABILITY" ]; then
  grep -F "setcap ${CAPABILITY} /usr/bin/${BIN}" "$tmp/deb-control/postinst" >/dev/null
else
  grep -F 'if [ -n "" ]; then' "$tmp/deb-control/postinst" >/dev/null
fi

test "$(rpm -qp --qf '%{NAME}' "$rpm")" = "$BIN"
test "$(rpm -qp --qf '%{VERSION}' "$rpm")" = "$VERSION"
test "$(rpm -qp --qf '%{RELEASE}' "$rpm")" = "$RPM_RELEASE"
test "$(rpm -qp --qf '%{ARCH}' "$rpm")" = "$RPM_ARCH"
rpm_requires="$(rpm -qpR "$rpm")"
if [ -n "$CAPABILITY" ]; then
  printf '%s' "$rpm_requires" | grep -F "libcap" >/dev/null
else
  if printf '%s' "$rpm_requires" | grep -F "libcap" >/dev/null; then
    echo "unexpected libcap dependency for package without capabilities" >&2
    exit 1
  fi
fi
rpm -qplv "$rpm" | grep -Eq "^-rwxr-xr-x +1 root +root +[0-9]+ .* /usr/bin/${BIN}$"
rpm -qp --scripts "$rpm" | grep -F "chmod 0755 /usr/bin/${BIN}" >/dev/null
if [ -n "$CAPABILITY" ]; then
  rpm -qp --scripts "$rpm" | grep -F "setcap ${CAPABILITY} /usr/bin/${BIN}" >/dev/null
else
  rpm -qp --scripts "$rpm" | grep -F 'if [ -n "" ]; then' >/dev/null
fi

echo "package checks passed for ${BIN} ${VERSION}"
