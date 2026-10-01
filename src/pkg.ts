import { Plugin , __sys__} from "xypriss";

export const pkg = Plugin.manifest<{
  name: string;
  version: string;
  description: string;
}>(__sys__);
