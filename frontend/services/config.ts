import { Platform } from "react-native";

// Centralized API configuration
export const getApiBaseUrl = (): string => {
  if (__DEV__) {
    // Android emulator uses 10.0.2.2 to reach host machine
    // iOS simulator and web use localhost
    if (Platform.OS === "android") {
      return "http://10.0.2.2:3000";
    }
    return "http://localhost:3000";
  }
  // Production URL - update this when you deploy
  return "https://api.your-domain.com";
};

export const API_BASE = getApiBaseUrl();
