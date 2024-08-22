import "./index.css";
import { invoke } from "@tauri-apps/api/core";
import {
  useQuery,
} from "@tanstack/react-query";
import Page from "./Page";
import { Link } from "@tanstack/react-router";

interface AppProps {
  block_id?: string;
}

const App: React.FC<AppProps> = ({ block_id }) => {

  const configuration = useQuery({
    queryKey: ["loadConfiguration"],
    queryFn: async () => {
      const response: any = await invoke("load_configuration_command");

      if (response.ok === false) {
        throw new Error(response.error)
      }

      const configuration = JSON.parse(response as string);

      const workspace_response: any = await invoke("get_block_command", { blockId: configuration.workspaceId });

      if (workspace_response.ok === false) {
        throw new Error(workspace_response.error)
      }

      const workspace = JSON.parse(workspace_response as string)

      const data = {
        configuration: configuration,
        workspace: workspace
      };

      console.debug("Loaded Configuration", data)

      return data
    },
  })

  if (configuration.isLoading) {
    return <></>
  }

  return (
    <div className="">
      {configuration.data &&
        <div className="flex flex-col min-h-screen max-h-screen">
          <Page id={block_id ?? configuration.data.workspace.block_contents.contents} />
          {/* <div className="flex-grow" /> */}
          <div className="underline text-right p-4"><Link to='/'>Home</Link></div>
        </div>
      }
    </div>
  );
}

export default App;
