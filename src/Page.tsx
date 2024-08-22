import React from 'react';
import BlockList from './BlockList';
import BlockInput from './BlockInput';
import { invoke } from "@tauri-apps/api/core";
import {
    useQuery,
} from "@tanstack/react-query";
import BlockDebugInformation from './components/BlockDebugInformation';

interface PageProps {
    id: string
}

async function fetchBlock({ queryKey }) {
    const [_key, { id }] = queryKey;
    console.debug("Trying to load block page", id);
    const response: any = await invoke("get_block_command", { blockId: id });

    if (response.ok === false) {
        throw new Error(response.error)
    }

    const data = JSON.parse(response as string);

    console.debug("Displayed Page", data);

    return data;
}

// This is really Block and the other is ChildBlock
const Page: React.FC<PageProps> = ({ id }) => {

    const query = useQuery({
        queryKey: ["displayedBlock", { id }],
        queryFn: fetchBlock,
    })

    if (query.isLoading) {
        return <></>
    }

    return (
        <div className='flex flex-col flex-grow'>
            {
                <div>
                    <div className="flex px-4 pt-8 pb-1 border-b">
                        <p className='text-2xl font-semibold'>{query.data.block_contents.contents}</p>
                    </div>
                    <div className='p-4'>
                        <BlockList blocks={query.data.children} />
                        <BlockInput parent={query.data} />
                    </div>
                </div>
            }
            <div className='p-4 mt-auto'>
                <BlockDebugInformation block={query.data} />
            </div>
        </div>
    )
}

export default Page;