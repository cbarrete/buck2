// @generated
// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: install.proto

// Protobuf Java Version: 3.25.6
package com.facebook.buck.install.model;

/**
 * <pre>
 * Should be kept in sync with buck2_error::ErrorTier.
 * </pre>
 *
 * Protobuf enum {@code install.ErrorCategory}
 */
@javax.annotation.Generated(value="protoc", comments="annotations:ErrorCategory.java.pb.meta")
public enum ErrorCategory
    implements com.google.protobuf.ProtocolMessageEnum {
  /**
   * <code>ERROR_CATEGORY_UNSPECIFIED = 0;</code>
   */
  ERROR_CATEGORY_UNSPECIFIED(0),
  /**
   * <pre>
   * Unexpected errors in installer. AKA Infra error.
   * </pre>
   *
   * <code>TIER_0 = 1;</code>
   */
  TIER_0(1),
  /**
   * <pre>
   * Expected errors in inputs explicitly tracked by buck. AKA User error.
   * </pre>
   *
   * <code>INPUT = 2;</code>
   */
  INPUT(2),
  /**
   * <pre>
   * Errors that may be triggered by issues with the host,
   * resource limits, non-explicit dependencies or potentially
   * ambiguous input errors.
   * These can be tracked but not eliminated.
   * </pre>
   *
   * <code>ENVIRONMENT = 3;</code>
   */
  ENVIRONMENT(3),
  UNRECOGNIZED(-1),
  ;

  /**
   * <code>ERROR_CATEGORY_UNSPECIFIED = 0;</code>
   */
  public static final int ERROR_CATEGORY_UNSPECIFIED_VALUE = 0;
  /**
   * <pre>
   * Unexpected errors in installer. AKA Infra error.
   * </pre>
   *
   * <code>TIER_0 = 1;</code>
   */
  public static final int TIER_0_VALUE = 1;
  /**
   * <pre>
   * Expected errors in inputs explicitly tracked by buck. AKA User error.
   * </pre>
   *
   * <code>INPUT = 2;</code>
   */
  public static final int INPUT_VALUE = 2;
  /**
   * <pre>
   * Errors that may be triggered by issues with the host,
   * resource limits, non-explicit dependencies or potentially
   * ambiguous input errors.
   * These can be tracked but not eliminated.
   * </pre>
   *
   * <code>ENVIRONMENT = 3;</code>
   */
  public static final int ENVIRONMENT_VALUE = 3;


  public final int getNumber() {
    if (this == UNRECOGNIZED) {
      throw new java.lang.IllegalArgumentException(
          "Can't get the number of an unknown enum value.");
    }
    return value;
  }

  /**
   * @param value The numeric wire value of the corresponding enum entry.
   * @return The enum associated with the given numeric wire value.
   * @deprecated Use {@link #forNumber(int)} instead.
   */
  @java.lang.Deprecated
  public static ErrorCategory valueOf(int value) {
    return forNumber(value);
  }

  /**
   * @param value The numeric wire value of the corresponding enum entry.
   * @return The enum associated with the given numeric wire value.
   */
  public static ErrorCategory forNumber(int value) {
    switch (value) {
      case 0: return ERROR_CATEGORY_UNSPECIFIED;
      case 1: return TIER_0;
      case 2: return INPUT;
      case 3: return ENVIRONMENT;
      default: return null;
    }
  }

  public static com.google.protobuf.Internal.EnumLiteMap<ErrorCategory>
      internalGetValueMap() {
    return internalValueMap;
  }
  private static final com.google.protobuf.Internal.EnumLiteMap<
      ErrorCategory> internalValueMap =
        new com.google.protobuf.Internal.EnumLiteMap<ErrorCategory>() {
          public ErrorCategory findValueByNumber(int number) {
            return ErrorCategory.forNumber(number);
          }
        };

  public final com.google.protobuf.Descriptors.EnumValueDescriptor
      getValueDescriptor() {
    if (this == UNRECOGNIZED) {
      throw new java.lang.IllegalStateException(
          "Can't get the descriptor of an unrecognized enum value.");
    }
    return getDescriptor().getValues().get(ordinal());
  }
  public final com.google.protobuf.Descriptors.EnumDescriptor
      getDescriptorForType() {
    return getDescriptor();
  }
  public static final com.google.protobuf.Descriptors.EnumDescriptor
      getDescriptor() {
    return com.facebook.buck.install.model.InstallerProto.getDescriptor().getEnumTypes().get(0);
  }

  private static final ErrorCategory[] VALUES = values();

  public static ErrorCategory valueOf(
      com.google.protobuf.Descriptors.EnumValueDescriptor desc) {
    if (desc.getType() != getDescriptor()) {
      throw new java.lang.IllegalArgumentException(
        "EnumValueDescriptor is not for this type.");
    }
    if (desc.getIndex() == -1) {
      return UNRECOGNIZED;
    }
    return VALUES[desc.getIndex()];
  }

  private final int value;

  private ErrorCategory(int value) {
    this.value = value;
  }

  // @@protoc_insertion_point(enum_scope:install.ErrorCategory)
}

